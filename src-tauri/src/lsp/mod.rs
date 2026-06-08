//! LSP 诊断集成 —— 文件编辑后的自动诊断注入
//!
//! 工作流程：
//! 1. 引擎执行 write_file/edit_file 工具后，自动触发 LSP 诊断
//! 2. 诊断结果以 system 消息形式注入回对话上下文
//! 3. AI 能感知到代码中的编译错误/警告并自动修复
//!
//! 架构：
//! - LspManager：管理多个 LSP 服务器连接
//! - StdioLspTransport：通过 stdio JSON-RPC 与 LSP 服务器通信
//! - 支持按文件语言自动选择对应的 LSP 服务器
//!
//! Ported from CodeWhale crates/tui/src/lsp/
//!
//! LSP integration: post-edit diagnostics injection.
//!
//! Ported from CodeWhale crates/tui/src/lsp/

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;

pub mod client;
pub mod diagnostics;
pub mod registry;

pub use client::{LspTransport, StdioLspTransport};
pub use diagnostics::{Diagnostic, DiagnosticBlock, Severity, render_blocks};
pub use registry::Language;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LspConfig {
    pub enabled: bool,
    pub poll_after_edit_ms: u64,
    pub max_diagnostics_per_file: usize,
    pub include_warnings: bool,
    pub servers: HashMap<String, Vec<String>>,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_after_edit_ms: 5_000,
            max_diagnostics_per_file: 20,
            include_warnings: false,
            servers: HashMap::new(),
        }
    }
}

impl LspConfig {
    fn resolve_command(&self, lang: Language) -> Option<(String, Vec<String>)> {
        if let Some(parts) = self.servers.get(lang.as_key()) {
            if let Some((first, rest)) = parts.split_first() {
                return Some((first.clone(), rest.to_vec()));
            }
        }
        let (cmd, args) = registry::server_for(lang)?;
        Some((cmd.to_string(), args.iter().map(|a| (*a).to_string()).collect()))
    }
}

pub struct LspManager {
    config: LspConfig,
    workspace: PathBuf,
    transports: AsyncMutex<HashMap<Language, Arc<dyn LspTransport>>>,
    missing_warned: AsyncMutex<HashSet<Language>>,
}

impl LspManager {
    pub fn new(config: LspConfig, workspace: PathBuf) -> Self {
        Self {
            config,
            workspace,
            transports: AsyncMutex::new(HashMap::new()),
            missing_warned: AsyncMutex::new(HashSet::new()),
        }
    }

    pub fn config(&self) -> &LspConfig {
        &self.config
    }

    pub async fn diagnostics_for(&self, file: &Path, _edit_seq: u64) -> Option<DiagnosticBlock> {
        if !self.config.enabled { return None; }
        let lang = registry::detect_language(file);
        if lang == Language::Other { return None; }

        let text = match tokio::fs::read_to_string(file).await {
            Ok(text) => text,
            Err(err) => {
                tracing::debug!(?err, file = %file.display(), "lsp: read file failed");
                return None;
            }
        };
        let transport = match self.transport_for(lang).await {
            Some(t) => t,
            None => return None,
        };

        let wait = Duration::from_millis(self.config.poll_after_edit_ms);
        let raw = match timeout(wait, transport.diagnostics_for(file, &text, wait)).await {
            Ok(Ok(items)) => items,
            Ok(Err(err)) => {
                tracing::debug!(?err, file = %file.display(), "lsp: diagnostics call failed");
                return None;
            }
            Err(_) => {
                tracing::debug!(file = %file.display(), "lsp: diagnostics timed out");
                return None;
            }
        };
        let mut items: Vec<Diagnostic> = raw.into_iter()
            .filter(|d| match d.severity {
                diagnostics::Severity::Error => true,
                diagnostics::Severity::Warning => self.config.include_warnings,
                _ => false,
            }).collect();
        items.sort_by_key(|d| match d.severity {
            diagnostics::Severity::Error => 0u8,
            diagnostics::Severity::Warning => 1u8,
            diagnostics::Severity::Information => 2u8,
            diagnostics::Severity::Hint => 3u8,
        });
        let mut block = DiagnosticBlock {
            file: relative_to_workspace(&self.workspace, file),
            items,
        };
        block.truncate(self.config.max_diagnostics_per_file);
        if block.items.is_empty() { None } else { Some(block) }
    }

    async fn transport_for(&self, lang: Language) -> Option<Arc<dyn LspTransport>> {
        if let Some(t) = self.transports.lock().await.get(&lang) {
            return Some(t.clone());
        }
        let (cmd, args) = self.config.resolve_command(lang)?;
        match StdioLspTransport::spawn(&cmd, &args, lang, self.workspace.clone()).await {
            Ok(transport) => {
                let arc: Arc<dyn LspTransport> = Arc::new(transport);
                self.transports.lock().await.insert(lang, arc.clone());
                Some(arc)
            }
            Err(err) => {
                tracing::warn!(language = %lang.as_key(), command = %cmd, error = %err, "lsp: server unavailable");
                None
            }
        }
    }

    pub async fn shutdown_all(&self) {
        let transports: Vec<Arc<dyn LspTransport>> =
            self.transports.lock().await.values().cloned().collect();
        for transport in transports {
            transport.shutdown().await;
        }
    }
}

impl LspManager {
    pub fn disabled() -> Self {
        Self::new(LspConfig { enabled: false, ..LspConfig::default() }, PathBuf::new())
    }
}

fn relative_to_workspace(workspace: &Path, path: &Path) -> PathBuf {
    if let Ok(rel) = path.strip_prefix(workspace) {
        return rel.to_path_buf();
    }
    PathBuf::from(
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("unknown")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeTransport {
        items: Vec<Diagnostic>,
        calls: AtomicUsize,
    }

    impl FakeTransport {
        fn new(items: Vec<Diagnostic>) -> Self {
            Self { items, calls: AtomicUsize::new(0) }
        }
        fn call_count(&self) -> usize {
            self.calls.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl LspTransport for FakeTransport {
        async fn diagnostics_for(&self, _path: &Path, _text: &str, _wait: Duration) -> anyhow::Result<Vec<Diagnostic>> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.items.clone())
        }
        async fn shutdown(&self) {}
    }

    #[tokio::test]
    async fn returns_none_when_disabled() {
        let mgr = LspManager::new(LspConfig { enabled: false, ..LspConfig::default() }, PathBuf::from("/tmp"));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("foo.rs");
        tokio::fs::write(&path, b"fn main() {}").await.unwrap();
        assert!(mgr.diagnostics_for(&path, 1).await.is_none());
    }

    #[tokio::test]
    async fn returns_none_for_unknown_language() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = LspManager::new(LspConfig::default(), dir.path().to_path_buf());
        let path = dir.path().join("notes.txt");
        tokio::fs::write(&path, b"hi").await.unwrap();
        assert!(mgr.diagnostics_for(&path, 1).await.is_none());
    }
}
