//! # Dream — 跨会话记忆整合子系统
//!
//! Dream 是 CodeWhale 的记忆整合系统，参照 Anthropic Claude Code
//! 的 KAIROS/Dream 机制设计。用户通过 `&dream` 命令手动触发，
//! 在后台启动一个受限子代理，对记忆目录中的文件进行四阶段整合
//!（Orient → Gather → Consolidate → Prune）。

pub mod config;
pub mod consolidator;
pub mod lock;
pub mod memdir;

use std::path::PathBuf;

use anyhow::Result;

pub use config::DreamConfig;
pub use lock::ConsolidationLock;

/// The Dream manager — stateless orchestrator.
#[derive(Debug, Clone)]
pub struct DreamManager {
    /// Resolved configuration snapshot.
    pub config: DreamConfig,
}

impl DreamManager {
    /// Create a new manager from a [`DreamConfig`].
    ///
    /// The config's `memory_dir` is resolved (tilde‑expanded) once
    /// here so callers don't have to repeat the expansion.
    pub fn new(config: DreamConfig) -> Self {
        Self { config }
    }
}

/// A consolidation task ready to be dispatched to a sub‑agent.
#[derive(Debug, Clone)]
pub struct DreamTask {
    /// Resolved memory directory path.
    pub memory_dir: PathBuf,
    /// System prompt for the sub‑agent.
    pub prompt: String,
    /// Tool allow‑list for the sub‑agent.
    pub allowed_tools: Vec<String>,
}

/// Create the `~/.codewhale/memories/` directory and a seed
/// `MEMORY.md` file if they don't exist.
///
/// `memory_dir` must be a tilde-resolved path (callers should pass
/// [`DreamConfig::resolved_memory_dir`]).
pub fn init_memory_dir(memory_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(memory_dir)?;

    let index_path = memory_dir.join("MEMORY.md");
    if !index_path.exists() {
        let seed = r#"# 记忆索引

## 主题文件

*暂无记忆文件。Dream 会在整合过程中自动创建和维护主题文件。*

上次更新: never
文件总数: 0
"#;
        std::fs::write(&index_path, seed)?;
    }

    Ok(())
}
