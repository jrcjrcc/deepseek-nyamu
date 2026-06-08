//! LSP 客户端 —— 基于 stdio 的轻量 JSON-RPC 实现
//!
//! 设计决策：不依赖 tower-lsp 库，因为 LSP 线路协议足够简单，
//! 自行实现在 ~400 LOC 内完成。
//!
//! 架构：
//! - LspTransport trait：LspManager 的通信接口抽象
//! - StdioLspTransport：通过 tokio::process::Command 启动 LSP 服务器，
//!   运行三个 tokio 任务（reader / writer / dispatcher）
//! - 使用 Content-Length 帧格式的 JSON-RPC 通过 stdin/stdout 通信
//!
//! 支持的语言服务器通过 registry::server_for() 静态注册。
//!
//! Ported from CodeWhale crates/tui/src/lsp/client.rs
//!
//! Thin JSON-RPC over stdio client for LSP servers.
//!
//! We deliberately do NOT depend on `tower-lsp`. The LSP wire protocol is
//! small enough that handling it ourselves is a self-contained ~400 LOC.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use super::diagnostics::{Diagnostic, Severity};
use super::registry::Language;

const MAX_FRAME_BYTES: usize = 1_048_576; // 1 MiB

/// Chunker for reading Content-Length framed JSON-RPC lines from stdout.
struct FrameReader<R> {
    reader: BufReader<R>,
    buf: Vec<u8>,
}

impl<R: AsyncReadExt + Unpin> FrameReader<R> {
    fn new(reader: R) -> Self {
        Self { reader: BufReader::new(reader), buf: Vec::with_capacity(4096) }
    }

    async fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        let mut content_length: Option<usize> = None;
        self.buf.clear();
        let mut header_buf = String::new();

        loop {
            header_buf.clear();
            let n = self.reader.read_line(&mut header_buf).await?;
            if n == 0 {
                return if content_length.is_some() {
                    Err(anyhow!("unexpected EOF in LSP body"))
                } else {
                    Ok(None)
                };
            }
            let line = header_buf.trim();
            if line.is_empty() {
                // End of headers
                break;
            }
            if let Some(val) = line
                .strip_prefix("Content-Length: ")
                .or_else(|| line.strip_prefix("content-length: "))
            {
                content_length = Some(val.trim().parse::<usize>().map_err(|_| anyhow!("invalid Content-Length: {val}"))?);
            }
        }

        let len = content_length.ok_or_else(|| anyhow!("missing Content-Length header"))?;
        if len > MAX_FRAME_BYTES {
            return Err(anyhow!("LSP frame too large: {len} > {MAX_FRAME_BYTES}"));
        }
        self.buf.resize(len, 0);
        self.reader.read_exact(&mut self.buf).await?;
        Ok(Some(std::mem::take(&mut self.buf)))
    }
}

/// Trait the LSP manager talks to.
#[async_trait]
pub trait LspTransport: Send + Sync {
    async fn diagnostics_for(&self, path: &Path, text: &str, wait: Duration) -> Result<Vec<Diagnostic>>;
    async fn shutdown(&self);
}

/// Stdio-backed transport. Spawns the LSP server as a child process.
pub struct StdioLspTransport {
    child: AsyncMutex<Option<Child>>,
    tx_outbound: mpsc::Sender<Vec<u8>>,
    diagnostics_rx: AsyncMutex<mpsc::Receiver<(PathBuf, Vec<Diagnostic>)>>,
    pending: Arc<AsyncMutex<HashMap<i64, oneshot::Sender<Value>>>>,
    next_id: AsyncMutex<i64>,
    language_id: &'static str,
    opened: AsyncMutex<HashMap<PathBuf, i64>>,
}

impl StdioLspTransport {
    /// Spawn `command args…` and run the LSP `initialize` handshake.
    pub async fn spawn(
        command: &str,
        args: &[String],
        language: Language,
        workspace: PathBuf,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd.spawn()
            .with_context(|| format!("failed to spawn LSP server `{command}`"))?;

        let stdin = child.stdin.take()
            .context("LSP child has no stdin handle")?;
        let stdout = child.stdout.take()
            .context("LSP child has no stdout handle")?;

        let (tx_outbound, rx_outbound) = mpsc::channel::<Vec<u8>>(64);
        let (tx_inbound, rx_inbound) = mpsc::channel::<Value>(64);
        let (tx_diag, rx_diag) = mpsc::channel::<(PathBuf, Vec<Diagnostic>)>(64);

        // Writer task
        let _outbound_handle = tokio::spawn(writer_task(stdin, rx_outbound));

        // Reader task → parse frames → push raw JSON
        let _reader_handle = tokio::spawn(reader_task(stdout, tx_inbound));

        // Dispatcher task → route to diagnostics or pending map
        let pending: Arc<AsyncMutex<HashMap<i64, oneshot::Sender<Value>>>> =
            Arc::new(AsyncMutex::new(HashMap::new()));
        let p = pending.clone();
        let _dispatcher_handle = tokio::spawn(dispatcher_task(rx_inbound, tx_diag, p));

        // Send `initialize` and `initialized`
        let init_payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": uri_from_path(&workspace),
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": { "relatedInformation": false }
                    }
                },
                "workspaceFolders": [{
                    "uri": uri_from_path(&workspace),
                    "name": "workspace"
                }]
            }
        });
        send_message(&tx_outbound, &init_payload).await?;

        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        send_message(&tx_outbound, &initialized).await?;

        Ok(Self {
            child: AsyncMutex::new(Some(child)),
            tx_outbound,
            diagnostics_rx: AsyncMutex::new(rx_diag),
            pending,
            next_id: AsyncMutex::new(2),
            language_id: language.language_id(),
            opened: AsyncMutex::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl LspTransport for StdioLspTransport {
    async fn diagnostics_for(&self, path: &Path, text: &str, wait: Duration) -> Result<Vec<Diagnostic>> {
        let path_buf = path.to_path_buf();
        let uri = uri_from_path(path);

        let mut opened = self.opened.lock().await;
        if opened.contains_key(path) {
            // Send didChange
            let change = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "version": 1
                    },
                    "contentChanges": [{
                        "text": text
                    }]
                }
            });
            send_message(&self.tx_outbound, &change).await?;
        } else {
            // Send didOpen
            opened.insert(path_buf.clone(), 1);
            drop(opened);
            let open = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": self.language_id,
                        "version": 1,
                        "text": text
                    }
                }
            });
            send_message(&self.tx_outbound, &open).await?;
        }

        // Wait for publishDiagnostics
        let deadline = tokio::time::Instant::now() + wait;
        let mut rx = self.diagnostics_rx.lock().await;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(Vec::new());
            }
            match timeout(remaining, rx.recv()).await {
                Ok(Some((file, diags))) if file == path_buf => return Ok(diags),
                Ok(Some(_)) => continue,
                Ok(None) => return Ok(Vec::new()),
                Err(_) => return Ok(Vec::new()),
            }
        }
    }

    async fn shutdown(&self) {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 999,
            "method": "shutdown",
            "params": null
        });
        let _ = send_message(&self.tx_outbound, &payload).await;
        let exit = json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        });
        let _ = send_message(&self.tx_outbound, &exit).await;
        let mut child = self.child.lock().await;
        if let Some(mut c) = child.take() {
            let _ = tokio::time::timeout(Duration::from_secs(2), c.wait()).await;
        }
    }
}

async fn writer_task(mut stdin: tokio::process::ChildStdin, mut rx: mpsc::Receiver<Vec<u8>>) {
    while let Some(bytes) = rx.recv().await {
        if stdin.write_all(&bytes).await.is_err() {
            break;
        }
        if stdin.flush().await.is_err() {
            break;
        }
    }
}

async fn reader_task(stdout: tokio::process::ChildStdout, tx: mpsc::Sender<Value>) {
    let mut frame_reader = FrameReader::new(stdout);
    loop {
        match frame_reader.next_frame().await {
            Ok(Some(frame)) => {
                if let Ok(value) = serde_json::from_slice::<Value>(&frame) {
                    let _ = tx.send(value).await;
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
}

async fn dispatcher_task(
    mut rx_inbound: mpsc::Receiver<Value>,
    tx_diag: mpsc::Sender<(PathBuf, Vec<Diagnostic>)>,
    pending: Arc<AsyncMutex<HashMap<i64, oneshot::Sender<Value>>>>,
) {
    while let Some(msg) = rx_inbound.recv().await {
        // Check if it's a publishDiagnostics notification
        if let Some("textDocument/publishDiagnostics") = msg.get("method").and_then(Value::as_str) {
            if let Some(params) = msg.get("params") {
                if let Some(uri) = params.get("uri").and_then(Value::as_str) {
                    if let Some(diags) = params.get("diagnostics").and_then(Value::as_array) {
                        let file = path_from_uri(uri).unwrap_or_else(|| PathBuf::from(uri));
                        let parsed: Vec<Diagnostic> = diags.iter().filter_map(|d| {
                            let range = d.get("range")?;
                            let start = range.get("start")?;
                            let line = start.get("line")?.as_i64()? as u32 + 1;
                            let column = start.get("character")?.as_i64()? as u32 + 1;
                            let severity = Severity::from_lsp(d.get("severity").and_then(Value::as_i64));
                            let message = d.get("message")?.as_str()?.to_string();
                            Some(Diagnostic {
                                line,
                                column,
                                severity: severity.unwrap_or(Severity::Error),
                                message,
                            })
                        }).collect();
                        let _ = tx_diag.send((file, parsed)).await;
                    }
                }
            }
            continue;
        }

        // Check if it's a response with an id
        if let Some(id_val) = msg.get("id") {
            if let Some(id) = id_val.as_i64() {
                let mut p = pending.lock().await;
                if let Some(sender) = p.remove(&id) {
                    let _ = sender.send(msg);
                }
            }
        }
    }
}

fn uri_from_path(path: &Path) -> String {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    format!("file:///{}", abs.display().to_string().replace('\\', "/").trim_start_matches("file:///"))
}

fn path_from_uri(uri: &str) -> Option<PathBuf> {
    let path_str = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .or_else(|| uri.strip_prefix("file:"))?;
    let path_str = if cfg!(windows) {
        path_str.trim_start_matches('/')
    } else {
        path_str
    };
    Some(PathBuf::from(path_str))
}

async fn send_message(tx: &mpsc::Sender<Vec<u8>>, payload: &Value) -> Result<()> {
    let body = serde_json::to_vec(payload)?;
    let len = body.len();
    let header = format!("Content-Length: {len}\r\n\r\n");
    let mut frame = header.into_bytes();
    frame.extend(body);
    tx.send(frame).await.map_err(|_| anyhow!("LSP writer channel closed"))?;
    Ok(())
}
