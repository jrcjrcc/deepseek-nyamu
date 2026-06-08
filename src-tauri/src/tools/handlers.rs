//! 工具处理器的具体实现模块。
//!
//! 本模块实现了 DeepWhale 系统中全部 13 个内置工具的具体逻辑，
//! 每个工具对应一个结构体，实现了 `ToolHandler` trait。
//!
//! # 工具列表
//!
//! | 工具名称 | 结构体 | 功能说明 | 是否写入操作 |
//! |----------|--------|----------|-------------|
//! | `read_file` | `ReadFileHandler` | 读取文件内容 | 否 |
//! | `write_file` | `WriteFileHandler` | 创建或覆盖文件 | 是 |
//! | `edit_file` | `EditFileHandler` | 搜索替换方式编辑文件 | 是 |
//! | `apply_patch` | `ApplyPatchHandler` | 应用 diff 补丁 | 是 |
//! | `exec_shell` | `ExecShellHandler` | 执行 Shell 命令 | 是 |
//! | `grep_files` | `GrepFilesHandler` | 正则搜索文件内容 | 否 |
//! | `file_search` | `FileSearchHandler` | 按文件名搜索 | 否 |
//! | `list_dir` | `ListDirHandler` | 列出目录内容 | 否 |
//! | `web_search` | `WebSearchHandler` | 网页搜索 | 否 |
//! | `fetch_url` | `FetchUrlHandler` | 获取网页内容 | 否 |
//! | `todo_write` | `TodoWriteHandler` | 写入待办事项 | 是 |
//! | `todo_list` | `TodoListHandler` | 列出待办事项 | 否 |
//! | `sub_agent` | `SubAgentHandler` | 创建子代理 | 否 |
//!
//! # 辅助函数
//! - `parse_args`：将 `ToolInvocation` 中的参数统一解析为 `serde_json::Value`
//! - `ok` / `err`：快捷构造成功或失败的 `ToolOutput`
//!
//! # 平台适配
//! - Windows 平台在 `exec_shell` 中使用 PowerShell，`grep_files` 使用 `findstr`
//! - Unix 平台使用 `sh`、`rg`（ripgrep）、`fd` 等原生工具
//! - 包含 Windows 系统代码页解码支持（`decode_stdout` 相关函数）
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nyamu_protocol::{ToolKind, ToolOutput, ToolPayload};
use nyamu_tools::{FunctionCallError, ToolHandler, ToolInvocation, required_str, optional_str};
use serde_json::Value;
use tokio::process::Command;
use crate::sandbox::{confine_command, global_sandbox_spec};

/// 将 `ToolInvocation` 中的参数负载统一解析为 `serde_json::Value`。
///
/// 根据负载类型的不同进行不同的处理：
/// - `Function`: 将 JSON 字符串反序列化为 Value
/// - `Custom`: 尝试 JSON 解析，失败则直接作为字符串 Value
/// - `LocalShell`: 将 `command`、`cwd`、`timeout_ms` 组合为 JSON 对象
/// - `Mcp`: 不支持，返回错误
///
/// # 参数
/// - `inv`：工具调用信息，包含负载和元数据
///
/// # 返回值
/// 成功返回 `Value`，失败返回 `FunctionCallError`。
fn parse_args(inv: &ToolInvocation) -> Result<Value, FunctionCallError> {
    match &inv.payload {
        ToolPayload::Function { arguments } => {
            serde_json::from_str(arguments).map_err(|e| FunctionCallError::ExecutionFailed {
                name: inv.tool_name.clone(),
                error: format!("invalid JSON: {e}"),
            })
        }
        ToolPayload::Custom { input } => {
            Ok(serde_json::from_str(input).unwrap_or(Value::String(input.clone())))
        }
        ToolPayload::LocalShell { params } => {
            Ok(serde_json::json!({"command": params.command, "cwd": params.cwd, "timeout_ms": params.timeout_ms}))
        }
        ToolPayload::Mcp { .. } => Err(FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(),
            error: "unexpected MCP payload".to_string(),
        }),
    }
}

/// 快捷构造一个成功的工具输出（`ToolOutput::Function`）。
///
/// # 参数
/// - `value`: 输出数据，会包装为 `body: Some(value)`, `success: true`。
fn ok(value: Value) -> std::result::Result<ToolOutput, FunctionCallError> {
    Ok(ToolOutput::Function { body: Some(value), success: true })
}

/// 快捷构造一个失败的工具输出（`ToolOutput::Function`）。
///
/// 注意：此处返回的是 `Ok(ToolOutput{ success: false })` 而非 `Err(...)`，
/// 即工具执行"成功返回了错误结果"。这种设计让调用者能统一处理输出，
/// 通过 `success` 字段判断执行是否成功。
///
/// # 参数
/// - `msg`: 错误消息，会作为字符串 Value 放在 body 中。
fn err(msg: &str) -> std::result::Result<ToolOutput, FunctionCallError> {
    Ok(ToolOutput::Function { body: Some(Value::String(msg.to_string())), success: false })
}

/// 返回 `read_file` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `path`（必填，string）：要读取的文件路径。
pub fn read_file_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string","description":"File path"}},"required":["path"]})
}
/// 返回 `write_file` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `path`（必填，string）：要写入的文件路径。
/// - `content`（必填，string）：文件内容。
pub fn write_file_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]})
}
/// 返回 `edit_file` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `path`（必填，string）：要编辑的文件路径。
/// - `search`（必填，string）：要搜索的文本。
/// - `replace`（必填，string）：替换后的文本。
pub fn edit_file_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"search":{"type":"string"},"replace":{"type":"string"}},"required":["path","search","replace"]})
}
/// 返回 `apply_patch` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `path`（必填，string）：要打补丁的文件路径。
/// - `patch`（必填，string）：diff 补丁内容。
pub fn apply_patch_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"patch":{"type":"string"}},"required":["path","patch"]})
}
/// 返回 `exec_shell` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `command`（必填，string）：要执行的 shell 命令。
/// - `cwd`（可选，string）：工作目录。
/// - `timeout_ms`（可选，integer）：超时时间（毫秒）。
pub fn exec_shell_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"command":{"type":"string"},"cwd":{"type":"string"},"timeout_ms":{"type":"integer"}},"required":["command"]})
}
/// 返回 `grep_files` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `pattern`（必填，string）：搜索的正则表达式模式。
/// - `path`（可选，string）：搜索起始路径。
/// - `max_results`（可选，integer）：最大结果数。
pub fn grep_files_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"pattern":{"type":"string"},"path":{"type":"string"},"max_results":{"type":"integer"}},"required":["pattern"]})
}
/// 返回 `file_search` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `query`（必填，string）：文件名搜索关键词。
/// - `limit`（可选，integer）：返回结果数量上限。
pub fn file_search_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"},"path":{"type":"string"},"timeout_ms":{"type":"integer","description":"Timeout in ms (default 30000)"}},"required":["query"]})
}
/// 返回 `list_dir` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `path`（必填，string）：要列出的目录路径。
pub fn list_dir_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]})
}
/// 返回 `web_search` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `query`（必填，string）：搜索关键词。
/// - `max_results`（可选，integer）：最大结果数。
pub fn web_search_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"query":{"type":"string"},"max_results":{"type":"integer"}},"required":["query"]})
}
/// 返回 `sub_agent` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `prompt`（必填，string）：交给子代理的任务描述。
/// - `model`（可选，string）：模型名称（如 `"deepseek-v4-flash"`）。
/// - `timeout_ms`（可选，integer）：超时时间，默认 120000 毫秒。
pub fn sub_agent_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"prompt":{"type":"string","description":"Task for sub-agent"},"model":{"type":"string","description":"Model override (optional)"},"timeout_ms":{"type":"integer","description":"Timeout in ms (default 120000)"}},"required":["prompt"]})
}
/// 返回 `fetch_url` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `url`（必填，string）：要获取的 URL 地址。
pub fn fetch_url_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"url":{"type":"string"}},"required":["url"]})
}
/// 返回 `todo_write` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `content`（必填，string）：待办事项内容。
/// - `status`（可选，string）：状态（如 `"pending"`）。
pub fn todo_write_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"content":{"type":"string"},"status":{"type":"string"}},"required":["content"]})
}
/// 返回 `todo_list` 工具的 JSON Schema 输入定义。
///
/// 参数：
/// - `status`（可选，string）：按状态过滤（如 `"pending"`），不传则返回全部。
pub fn todo_list_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"status":{"type":"string"}},"required":[]})
}

/// `read_file` 工具处理器：读取指定文件的文本内容。
///
/// 功能：
/// - 接收文件路径，通过沙箱模块解析为安全路径
/// - 使用 `tokio::fs::read_to_string` 异步读取
/// - 返回文件内容及实际路径
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct ReadFileHandler;
#[async_trait]
impl ToolHandler for ReadFileHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let path = crate::sandbox::resolve_path(&PathBuf::from(required_str(&args, "path").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?));
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => ok(serde_json::json!({"content": content, "path": path.to_string_lossy()})),
            Err(e) => err(&format!("Failed to read '{}': {e}", path.display())),
        }
    }
}

/// `write_file` 工具处理器：创建新文件或覆盖已有文件。
///
/// 功能：
/// - 接收文件路径和内容
/// - 自动创建父目录（如果不存在）
/// - 同步写入文件（`std::fs::write`）
/// - 返回写入路径和文件大小
///
/// 这是一个写操作（`is_mutating() -> true`）。
pub struct WriteFileHandler;
#[async_trait]
impl ToolHandler for WriteFileHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let path = crate::sandbox::resolve_path(&PathBuf::from(required_str(&args, "path").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?));
        let content = required_str(&args, "content").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?;
        if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
        match std::fs::write(&path, content) {
            Ok(_) => ok(serde_json::json!({"path": path.to_string_lossy(), "size": content.len()})),
            Err(e) => err(&format!("Failed to write '{}': {e}", path.display())),
        }
    }
}

/// `edit_file` 工具处理器：通过搜索替换方式编辑已有文件。
///
/// 功能：
/// - 接收文件路径、搜索文本、替换文本
/// - 在文件中查找第一个匹配的 `search` 字符串并替换为 `replace`
/// - 如果未找到搜索文本则返回错误
///
/// 这是一个写操作（`is_mutating() -> true`）。
pub struct EditFileHandler;
#[async_trait]
impl ToolHandler for EditFileHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let path = crate::sandbox::resolve_path(&PathBuf::from(required_str(&args, "path").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?));
        let search = required_str(&args, "search").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?;
        let replace = required_str(&args, "replace").map_err(|e| FunctionCallError::ExecutionFailed {
            name: inv.tool_name.clone(), error: e.to_string(),
        })?;
        let content = match std::fs::read_to_string(&path) { Ok(c) => c, Err(e) => return err(&format!("read failed: {e}")), };
        if let Some(pos) = content.find(search) {
            let new = format!("{}{}{}", &content[..pos], replace, &content[pos + search.len()..]);
            match std::fs::write(&path, &new) { Ok(_) => ok(serde_json::json!({"replaced": 1})), Err(e) => err(&format!("write failed: {e}")), }
        } else { err(&format!("not found in '{}'", path.display())) }
    }
}

/// `apply_patch` 工具处理器：应用结构化 diff 补丁到文件。
///
/// 功能：
/// - 接收补丁内容，写入临时文件
/// - 调用 `git apply --ignore-space-change` 应用补丁
/// - 在沙箱限制下执行（通过 `confine_command`）
/// - 如果 `git apply` 失败，建议用户改用 `write_file` + `edit_file`
///
/// 这是一个写操作（`is_mutating() -> true`）。
/// 依赖系统安装有 `git` 命令。
pub struct ApplyPatchHandler;
#[async_trait]
impl ToolHandler for ApplyPatchHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let patch = required_str(&args, "patch").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let tmp = std::env::temp_dir().join("dw_patch.diff");
        let _ = std::fs::write(&tmp, patch);
        let mut cmd = Command::new("git");
        confine_command(&mut cmd, global_sandbox_spec());
        cmd.arg("apply");
        cmd.arg("--ignore-space-change");
        cmd.arg(tmp.to_str().unwrap());
        if let Some(ws) = crate::sandbox::get_workspace() {
            cmd.current_dir(ws);
        }
        match cmd.output().await {
            Ok(o) if o.status.success() => { let _ = std::fs::remove_file(&tmp); ok(serde_json::json!({"status":"applied"})) }
            Ok(o) => {
                let _ = std::fs::remove_file(&tmp);
                let stderr = String::from_utf8_lossy(&o.stderr);
                err(&format!("Patch failed (try write_file+edit_file instead): {}", stderr))
            }
            Err(e) => { let _ = std::fs::remove_file(&tmp); err(&format!("git apply error: {e}. Use write_file + edit_file as fallback.")) }
        }
    }
}

/// 解码命令输出的字节流为字符串。
///
/// 优先使用 UTF-8 解码，失败时：
/// - Windows 平台：尝试使用系统代码页（如 GBK）重新解码
/// - 其他平台：使用 `String::from_utf8_lossy` 保留有效字符
fn decode_stdout(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            // On Windows, try decoding with system code page via encoding_rs
            #[cfg(windows)]
            {
                if let Some(cp) = get_system_codepage() {
                    if let Some(decoded) = decode_codepage(bytes, cp) {
                        return decoded;
                    }
                }
            }
            String::from_utf8_lossy(bytes).to_string()
        }
    }
}

/// Windows API 函数声明，用于获取系统代码页和多字节转宽字符转换。
#[cfg(windows)]
unsafe extern "system" {
    /// 获取当前系统的 ANSI 代码页标识符（如 936 代表 GBK）。
    fn GetACP() -> u32;
    /// 将多字节字符串转换为宽字符（UTF-16）字符串。
    fn MultiByteToWideChar(CodePage: u32, dwFlags: u32, lpMultiByteStr: *const i8, cbMultiByte: i32, lpWideCharStr: *mut u16, cchWideChar: i32) -> i32;
}

/// 获取 Windows 系统代码页编号，若为 UTF-8 (65001) 则返回 None。
#[cfg(windows)]
fn get_system_codepage() -> Option<u32> {
    unsafe {
        let cp = GetACP();
        if cp != 65001 { Some(cp) } else { None }
    }
}

/// 使用指定代码页解码字节流为 UTF-16，再转为 Rust String。
#[cfg(windows)]
fn decode_codepage(bytes: &[u8], codepage: u32) -> Option<String> {
    use std::ptr;
    unsafe {
        let wide_len = MultiByteToWideChar(codepage, 0, bytes.as_ptr() as *const i8, bytes.len() as i32, ptr::null_mut(), 0);
        if wide_len <= 0 { return None; }
        let mut wide_buf = vec![0u16; wide_len as usize];
        let result = MultiByteToWideChar(codepage, 0, bytes.as_ptr() as *const i8, bytes.len() as i32, wide_buf.as_mut_ptr(), wide_len);
        if result <= 0 { return None; }
        String::from_utf16(&wide_buf).ok()
    }
}

/// `exec_shell` 工具处理器：执行 Shell 命令。
///
/// 功能：
/// - 接收命令字符串、可选工作目录和超时时间
/// - Windows 平台：将命令写入临时 `.ps1` 文件，用 PowerShell 执行
/// - Unix 平台：使用 `sh -c` 直接执行
/// - 命令在沙箱限制下运行（`confine_command`）
/// - 结果包含退出码、标准输出和标准错误
///
/// 这是一个写操作（`is_mutating() -> true`）。
/// 默认超时 30 秒，可通过 `timeout_ms` 参数调整。
pub struct ExecShellHandler;
#[async_trait]
impl ToolHandler for ExecShellHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let timeout = optional_str(&args, "timeout_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(30000);
        let command_str = required_str(&args, "command").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let mut cmd: Command;
        let _script_path;
        #[cfg(windows)]
        {
            _script_path = std::env::temp_dir().join(format!("dw_{}.ps1", uuid::Uuid::new_v4()));
            let _ = std::fs::write(&_script_path, command_str);
            cmd = Command::new("powershell");
            cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", &_script_path.to_string_lossy()]);
            crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        }
        #[cfg(not(windows))]
        {
            cmd = Command::new("sh");
            cmd.arg("-c").arg(command_str);
        }
        if let Some(d) = cwd { cmd.current_dir(d); }
        let result = tokio::time::timeout(Duration::from_millis(timeout), cmd.output()).await;
        #[cfg(windows)] { let _ = std::fs::remove_file(&_script_path); }
        match result {
            Ok(Ok(o)) => ok(serde_json::json!({"exit_code":o.status.code().unwrap_or(-1),"stdout":decode_stdout(&o.stdout),"stderr":decode_stdout(&o.stderr)})),
            Ok(Err(e)) => err(&format!("Command failed: {e}")), Err(_) => err("Command timed out"),
        }
    }
}

/// `grep_files` 工具处理器：使用正则表达式搜索文件内容。
///
/// 功能：
/// - 接收正则模式、搜索路径和最大结果数
/// - Windows 平台：使用 `findstr /n /s` 命令
/// - Unix 平台：使用 `rg`（ripgrep）命令
/// - 结果截断至 32000 字符以防止过大输出
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct GrepFilesHandler;
#[async_trait]
impl ToolHandler for GrepFilesHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let path = crate::sandbox::resolve_path(&PathBuf::from(required_str(&args, "path").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?));
        let pattern = required_str(&args, "pattern").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let max = optional_str(&args, "max_results").and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);
        #[cfg(windows)]
        {
            // Windows: use findstr built-in
            let file_path = path.to_str().unwrap_or(".");
            let mut cmd = Command::new("cmd");
            crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
            cmd.args(["/C", &format!("findstr /n /s \"{pattern}\" \"{file_path}\\*\" 2>nul")]);
            match cmd.output().await {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout);
                    let lines: Vec<&str> = out.lines().take(max).collect();
                    let t = lines.join("\n");
                    let truncated = if t.len() > 32000 { format!("{}... [truncated]", &t[..32000]) } else { t };
                    ok(serde_json::json!({"matches":truncated,"total":lines.len()}))
                }
                Err(e) => err(&format!("findstr failed: {e}")),
            }
        }
        #[cfg(not(windows))]
        {
            // Unix: use ripgrep (rg)
            let mut cmd = Command::new("rg");
            cmd.args(["-n", "--max-count", &max.to_string(), pattern, path.to_str().unwrap_or(".")]);
            match cmd.output().await {
                Ok(o) => { let out = String::from_utf8_lossy(&o.stdout); let t = if out.len() > 32000 { format!("{}... [truncated]", &out[..32000]) } else { out.to_string() }; ok(serde_json::json!({"matches":t})) }
                Err(e) => err(&format!("grep failed: {e}")),
            }
        }
    }
}

/// `file_search` 工具处理器：按文件名搜索文件。
///
/// 功能：
/// - 接收文件名查询和可选限制数
/// - Windows 平台：使用 `dir /s /b | findstr /r` 管道
/// - Unix 平台：使用 `fd` 命令
/// - 支持正则表达式模式匹配文件名
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct FileSearchHandler;
#[async_trait]
impl ToolHandler for FileSearchHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let q = required_str(&args, "query").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let lim = optional_str(&args, "limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(20);
        let p = optional_str(&args, "path").unwrap_or(".").to_string();
        let timeout = optional_str(&args, "timeout_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(30000);
        #[cfg(windows)]
        {
            let mut cmd = Command::new("cmd");
            crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
            cmd.args(["/C", &format!("dir /s /b \"{}\" 2>nul | findstr /r \"{}\" 2>nul", p, q)]);
            match tokio::time::timeout(Duration::from_millis(timeout), cmd.output()).await {
                Ok(Ok(o)) => { let out = String::from_utf8_lossy(&o.stdout); let f: Vec<&str> = out.lines().take(lim).collect(); ok(serde_json::json!({"files":f,"total":f.len()})) }
                Ok(Err(e)) => err(&format!("file search failed: {e}")),
                Err(_) => err("file search timed out"),
            }
        }
        #[cfg(not(windows))]
        {
            let mut cmd = Command::new("fd"); cmd.args([&q, ".", "--max-results", &lim.to_string()]);
            match tokio::time::timeout(Duration::from_millis(timeout), cmd.output()).await {
                Ok(Ok(o)) => { let out = String::from_utf8_lossy(&o.stdout); let f: Vec<String> = out.lines().map(String::from).collect(); ok(serde_json::json!({"files":f,"total":f.len()})) }
                Ok(Err(_)) => err("file search failed (fd not found?)"),
                Err(_) => err("file search timed out"),
            }
        }
    }
}

/// `list_dir` 工具处理器：列出目录内容。
///
/// 功能：
/// - 接收目录路径（可选，默认当前目录）
/// - 通过沙箱解析为安全路径
/// - 异步读取目录条目，区分文件和子目录
/// - 返回路径和条目列表（含名称和类型）
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct ListDirHandler;
#[async_trait]
impl ToolHandler for ListDirHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let p = optional_str(&args, "path").unwrap_or(".");
        let path = crate::sandbox::resolve_path(&PathBuf::from(p));
        match tokio::fs::read_dir(&path).await {
            Ok(mut rd) => {
                let mut e = Vec::new();
                while let Ok(Some(entry)) = rd.next_entry().await {
                    let k = if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) { "dir" } else { "file" };
                    e.push(serde_json::json!({"name": entry.file_name().to_string_lossy(), "kind": k}));
                }
                ok(serde_json::json!({"path":path.to_string_lossy(),"entries":e}))
            }
            Err(e) => err(&format!("list_dir failed: {e}")),
        }
    }
}

/// `web_search` 工具处理器：执行网页搜索。
///
/// 功能：
/// - 接收搜索查询和可选的最大结果数
/// - 委托给 `crate::tools::web_search_simple()` 函数
/// - 使用 DuckDuckGo Lite 的免费搜索服务
/// - 返回标题和 URL 列表
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct WebSearchHandler;
#[async_trait]
impl ToolHandler for WebSearchHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let q = required_str(&args, "query").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        match crate::tools::web_search_simple(q).await {
            Ok(r) => ok(serde_json::json!({"results": r})), Err(e) => err(&format!("Web search: {e}")),
        }
    }
}

/// `sub_agent` 工具处理器：创建子代理执行独立任务。
///
/// 功能：
/// - 接收提示词（prompt）、可选模型和超时时间
/// - 从环境变量或配置中读取 DeepSeek API Key
/// - 调用 DeepSeek Chat API 完成独立对话
/// - 返回子代理的回复内容
///
/// 这是一个只读操作（`is_mutating() -> false`）。
/// 需要配置 `DEEPSEEK_API_KEY` 环境变量或 `config.toml`。
pub struct SubAgentHandler;
#[async_trait]
impl ToolHandler for SubAgentHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let prompt = required_str(&args, "prompt").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let model = optional_str(&args, "model").unwrap_or("deepseek-v4-flash");
        let timeout_ms = optional_str(&args, "timeout_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(120000);

        let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        let api_key = if api_key.is_empty() {
            nyamu_config::ConfigStore::load(None).ok()
                .and_then(|s| s.config.api_key)
                .unwrap_or_default()
        } else { api_key };
        if api_key.is_empty() { return err("No API key found. Set DEEPSEEK_API_KEY or configure in config.toml"); }

        let base_url = std::env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com/beta".to_string());

        let client = match reqwest::Client::builder().timeout(Duration::from_millis(timeout_ms)).build() {
            Ok(c) => c, Err(e) => return err(&format!("HTTP client: {e}")),
        };
        let body = serde_json::json!({"model": model, "messages": [{"role": "user", "content": prompt}], "max_tokens": 4096, "stream": false});
        match client.post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body).send().await
        {
            Ok(r) if r.status().is_success() => {
                match r.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                        ok(serde_json::json!({"response": content, "model": model}))
                    }
                    Err(e) => err(&format!("JSON parse: {e}")),
                }
            }
            Ok(r) => err(&format!("HTTP {}", r.status())),
            Err(e) => err(&format!("Request: {e}")),
        }
    }
}

/// `fetch_url` 工具处理器：获取指定 URL 的网页内容。
///
/// 功能：
/// - 接收 URL 地址
/// - 使用 `reqwest` 发送 HTTP GET 请求（15 秒超时）
/// - 返回 HTTP 状态码和响应体文本
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct FetchUrlHandler;
#[async_trait]
impl ToolHandler for FetchUrlHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let url = required_str(&args, "url").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let client = reqwest::Client::builder().timeout(Duration::from_secs(15)).user_agent("deepwhale").build().map_err(|e| FunctionCallError::ExecutionFailed { name: "fetch_url".into(), error: e.to_string() })?;
        match client.get(url).send().await {
            Ok(r) => { let s = r.status().as_u16(); match r.text().await { Ok(t) => ok(serde_json::json!({"status":s,"body":t})), Err(e) => err(&format!("read failed: {e}")), } }
            Err(e) => err(&format!("HTTP failed: {e}")),
        }
    }
}

/// 全局待办事项存储（进程内内存存储）。
///
/// 使用 `Mutex<Vec<(content, status)>>` 保证线程安全。
/// 每个元素是一个元组：`(内容, 状态)`。
/// 注意：此存储为内存级，程序重启后所有数据丢失。
static TODOS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// `todo_write` 工具处理器：添加一条待办事项。
///
/// 功能：
/// - 接收待办内容（content）和可选状态（status，默认 `"pending"`）
/// - 写入全局静态 `TODOS` 列表（进程内存储）
/// - 返回添加的内容和状态
///
/// 这是一个写操作（`is_mutating() -> true`）。
/// 注意：待办事项仅存在于内存中，程序重启后丢失。
pub struct TodoWriteHandler;
#[async_trait]
impl ToolHandler for TodoWriteHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let c = required_str(&args, "content").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let s = optional_str(&args, "status").unwrap_or("pending");
        TODOS.lock().unwrap().push((c.to_string(), s.to_string()));
        ok(serde_json::json!({"added":c,"status":s}))
    }
}

/// `todo_list` 工具处理器：列出待办事项。
///
/// 功能：
/// - 可选地按状态过滤（如只显示 `"pending"` 的项）
/// - 从全局静态 `TODOS` 列表中读取
/// - 返回带编号的待办事项列表
///
/// 这是一个只读操作（`is_mutating() -> false`）。
pub struct TodoListHandler;
#[async_trait]
impl ToolHandler for TodoListHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let f = optional_str(&args, "status");
        let todos = TODOS.lock().unwrap();
        let items: Vec<Value> = todos.iter().enumerate().filter_map(|(i, (c, s))| {
            if let Some(f) = f { if s != f { return None; } }
            Some(serde_json::json!({"id":i,"content":c,"status":s}))
        }).collect();
        ok(serde_json::json!({"todos":items,"total":items.len()}))
    }
}

// ═══════════════════════════════════════════════════════════════════
// 新增工具处理器：Git 操作类
// ═══════════════════════════════════════════════════════════════════

/// `git_status` 工具处理器：显示工作区 git 状态（porcelain v1 格式）
pub struct GitStatusHandler;
#[async_trait]
impl ToolHandler for GitStatusHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let cwd = optional_str(&args, "path").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("git");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["status", "--porcelain"]);
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({"status": decode_stdout(&o.stdout), "exit_code": o.status.code().unwrap_or(-1)})),
            Err(e) => err(&format!("git status failed: {e}")),
        }
    }
}
pub fn git_status_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"path":{"type":"string","description":"Optional working directory"}},"required":[]})
}

/// `git_diff` 工具处理器：显示文件差异
pub struct GitDiffHandler;
#[async_trait]
impl ToolHandler for GitDiffHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let staged = optional_str(&args, "staged").unwrap_or("false") == "true";
        let path = optional_str(&args, "path");
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("git");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        if staged { cmd.arg("diff").arg("--cached"); } else { cmd.arg("diff"); }
        if let Some(p) = path { cmd.arg(p); }
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({"diff": decode_stdout(&o.stdout), "exit_code": o.status.code().unwrap_or(-1)})),
            Err(e) => err(&format!("git diff failed: {e}")),
        }
    }
}
pub fn git_diff_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"staged":{"type":"boolean","description":"Show staged changes"},"path":{"type":"string","description":"Optional file path"},"cwd":{"type":"string","description":"Working directory"}},"required":[]})
}

/// `git_log` 工具处理器：显示提交历史
pub struct GitLogHandler;
#[async_trait]
impl ToolHandler for GitLogHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let count = optional_str(&args, "count").and_then(|s| s.parse::<usize>().ok()).unwrap_or(20);
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("git");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["log", "--oneline", &format!("-{}", count)]);
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({"log": decode_stdout(&o.stdout), "exit_code": o.status.code().unwrap_or(-1)})),
            Err(e) => err(&format!("git log failed: {e}")),
        }
    }
}
pub fn git_log_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"count":{"type":"integer","description":"Number of commits to show"},"cwd":{"type":"string","description":"Working directory"}},"required":[]})
}

/// `git_show` 工具处理器：显示某次提交的详细信息
pub struct GitShowHandler;
#[async_trait]
impl ToolHandler for GitShowHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let commit = required_str(&args, "commit").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("git");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["show", commit]);
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({"commit": decode_stdout(&o.stdout), "exit_code": o.status.code().unwrap_or(-1)})),
            Err(e) => err(&format!("git show failed: {e}")),
        }
    }
}
pub fn git_show_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"commit":{"type":"string","description":"Commit hash or ref"},"cwd":{"type":"string"}},"required":["commit"]})
}

/// `git_blame` 工具处理器：文件逐行归属标注
pub struct GitBlameHandler;
#[async_trait]
impl ToolHandler for GitBlameHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let file = required_str(&args, "file").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("git");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["blame", file]);
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({"blame": decode_stdout(&o.stdout), "exit_code": o.status.code().unwrap_or(-1)})),
            Err(e) => err(&format!("git blame failed: {e}")),
        }
    }
}
pub fn git_blame_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"file":{"type":"string","description":"File path"},"cwd":{"type":"string"}},"required":["file"]})
}

// ═══════════════════════════════════════════════════════════════════
// 新增工具处理器：GitHub 操作类
// ═══════════════════════════════════════════════════════════════════

/// `github_issue_context` 工具：查看 GitHub Issue 详情
pub struct GitHubIssueHandler;
#[async_trait]
impl ToolHandler for GitHubIssueHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let repo = required_str(&args, "repo").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let number = required_str(&args, "number").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let mut cmd = Command::new("gh");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["issue", "view", &format!("{}#{}", repo, number), "--json", "title,body,state,comments,author,createdAt,labels"]);
        match cmd.output().await {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout).to_string();
                ok(serde_json::json!({"issue": output, "repo": repo, "number": number}))
            }
            Ok(o) => err(&format!("gh issue view failed: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => err(&format!("gh command failed: {e}")),
        }
    }
}
pub fn github_issue_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"repo":{"type":"string","description":"Repository (user/repo)"},"number":{"type":"string","description":"Issue number"}},"required":["repo","number"]})
}

/// `github_pr_context` 工具：查看 GitHub PR 详情
pub struct GitHubPRHandler;
#[async_trait]
impl ToolHandler for GitHubPRHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let repo = required_str(&args, "repo").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let number = required_str(&args, "number").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let mut cmd = Command::new("gh");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["pr", "view", &format!("{}#{}", repo, number), "--json", "title,body,state,author,headRefName,baseRefName,additions,deletions,files,createdAt,comments,reviews"]);
        match cmd.output().await {
            Ok(o) if o.status.success() => {
                let output = String::from_utf8_lossy(&o.stdout).to_string();
                ok(serde_json::json!({"pr": output, "repo": repo, "number": number}))
            }
            Ok(o) => err(&format!("gh pr view failed: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => err(&format!("gh command failed: {e}")),
        }
    }
}
pub fn github_pr_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"repo":{"type":"string"},"number":{"type":"string"}},"required":["repo","number"]})
}

/// `github_comment` 工具：在 Issue/PR 上评论
pub struct GitHubCommentHandler;
#[async_trait]
impl ToolHandler for GitHubCommentHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let repo = required_str(&args, "repo").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let number = required_str(&args, "number").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let body = required_str(&args, "body").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let item_type = optional_str(&args, "type").unwrap_or("issue");
        let mut cmd = Command::new("gh");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args([item_type, "comment", &format!("{}#{}", repo, number), "--body", body]);
        match cmd.output().await {
            Ok(o) if o.status.success() => ok(serde_json::json!({"status": "commented"})),
            Ok(o) => err(&format!("Comment failed: {}", String::from_utf8_lossy(&o.stderr))),
            Err(e) => err(&format!("gh command failed: {e}")),
        }
    }
}
pub fn github_comment_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"repo":{"type":"string"},"number":{"type":"string"},"body":{"type":"string"},"type":{"type":"string","description":"issue or pr"}},"required":["repo","number","body"]})
}

// ═══════════════════════════════════════════════════════════════════
// 新增工具处理器：通知 / 记忆 / 诊断 / 撤销
// ═══════════════════════════════════════════════════════════════════

/// `notify` 工具处理器：发送桌面通知
pub struct NotifyHandler;
#[async_trait]
impl ToolHandler for NotifyHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let title = optional_str(&args, "title").unwrap_or("DeepWhale");
        let message = required_str(&args, "message").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        // Use notify-rust for desktop notification
        #[cfg(not(target_os = "linux"))]
        {
            let _ = notify_rust::Notification::new()
                .summary(title)
                .body(message)
                .appname("DeepWhale")
                .timeout(notify_rust::Timeout::Milliseconds(5000))
                .show();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("notify-send")
                .arg(title).arg(message)
                .output();
        }
        ok(serde_json::json!({"notified": true, "title": title, "message": message}))
    }
}
pub fn notify_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"title":{"type":"string","description":"Notification title"},"message":{"type":"string","description":"Notification body"}},"required":["message"]})
}

/// `remember` 工具处理器：持久化记录记忆
pub struct RememberHandler;
#[async_trait]
impl ToolHandler for RememberHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let note = required_str(&args, "note").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let memory_path = crate::memory::resolve_global_memory_path(None);
        crate::memory::quick_capture(&memory_path, note);
        ok(serde_json::json!({"recorded": true, "note": note}))
    }
}
pub fn remember_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"note":{"type":"string","description":"One-sentence memory to record"}},"required":["note"]})
}

/// `diagnostics` 工具处理器：显示工作区诊断信息
pub struct DiagnosticsHandler;
#[async_trait]
impl ToolHandler for DiagnosticsHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let _args = parse_args(&inv)?;
        let ws = crate::sandbox::get_workspace().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let mut diag = serde_json::json!({
            "workspace": ws,
            "os": std::env::consts::OS,
            "sandbox_mode": crate::sandbox::global_sandbox_spec().mode,
        });
        // Try to get git version
        if let Ok(o) = Command::new("git").arg("--version").output().await {
            if o.status.success() {
                diag["git_version"] = Value::String(String::from_utf8_lossy(&o.stdout).trim().to_string());
            }
        }
        // Try to get rustc version
        if let Ok(o) = Command::new("rustc").arg("--version").output().await {
            if o.status.success() {
                diag["rustc_version"] = Value::String(String::from_utf8_lossy(&o.stdout).trim().to_string());
            }
        }
        // Check if in git repo
        if !ws.is_empty() {
            if let Ok(o) = Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).current_dir(&ws).output().await {
                diag["in_git_repo"] = Value::String(String::from_utf8_lossy(&o.stdout).trim().to_string());
            }
        }
        ok(diag)
    }
}
pub fn diagnostics_schema() -> Value {
    serde_json::json!({"type":"object","properties":{},"required":[]})
}

/// `revert_turn` 工具处理器：通过快照回滚工作区变更
pub struct RevertTurnHandler;
#[async_trait]
impl ToolHandler for RevertTurnHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let turn_str = optional_str(&args, "turn");
        let ws_path = crate::sandbox::get_workspace();
        if ws_path.is_none() {
            return err("No workspace set. Set workspace to use revert_turn.");
        }
        let ws = ws_path.unwrap();
        // Try loading snapshot manager from engine or create temporary
        let sm = crate::snapshot::SnapshotManager::new(true);
        if let Some(ts) = turn_str {
            let snapshot_id = crate::snapshot::SnapshotId::new(ts.to_string());
            let _ = sm.restore(&ws, &snapshot_id).await;
            ok(serde_json::json!({"reverted": true, "turn": ts, "workspace": ws.to_string_lossy().to_string()}))
        } else {
            // List available snapshots
            let snapshots = sm.list_snapshots(&ws).await;
            let snapshot_list: Vec<Value> = snapshots.iter().map(|s| {
                serde_json::json!({"id": s.id.sha, "label": s.label, "timestamp": s.timestamp})
            }).collect();
            ok(serde_json::json!({"snapshots": snapshot_list, "workspace": ws.to_string_lossy().to_string()}))
        }
    }
}
pub fn revert_turn_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"turn":{"type":"integer","description":"Turn number to revert to. Omit to list snapshots."}},"required":[]})
}

// ═══════════════════════════════════════════════════════════════════
// 新增工具处理器：任务/计划管理
// ═══════════════════════════════════════════════════════════════════

/// 内存中的计划状态（进程内）
static PLAN_STATE: std::sync::Mutex<Vec<serde_json::Value>> = std::sync::Mutex::new(Vec::new());

/// 获取所有计划（供 Tauri 命令调用）
pub fn get_all_plans() -> Vec<serde_json::Value> {
    PLAN_STATE.lock().unwrap().clone()
}

/// 获取最新计划（供 Tauri 命令调用）
pub fn get_latest_plan() -> Option<serde_json::Value> {
    PLAN_STATE.lock().unwrap().last().cloned()
}

/// `update_plan` 工具处理器：更新会话级计划
pub struct UpdatePlanHandler;
#[async_trait]
impl ToolHandler for UpdatePlanHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let action = required_str(&args, "action").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        match action {
            "init" | "create" => {
                let title = optional_str(&args, "title").unwrap_or("Plan");
                let steps: Vec<Value> = args.get("steps")
                    .and_then(|s| s.as_array().cloned())
                    .unwrap_or_default();
                let plan = serde_json::json!({
                    "id": uuid::Uuid::new_v4().to_string(),
                    "title": title,
                    "steps": steps,
                    "current_step": 0,
                    "status": "active",
                });
                let mut state = PLAN_STATE.lock().unwrap();
                state.push(plan.clone());
                ok(serde_json::json!({"plan": plan, "total_plans": state.len()}))
            }
            "update" => {
                let step_index = optional_str(&args, "step_index").and_then(|s| s.parse::<usize>().ok());
                let status = optional_str(&args, "status").unwrap_or("in_progress");
                let notes = optional_str(&args, "notes");
                let mut state = PLAN_STATE.lock().unwrap();
                if let Some(plan) = state.last_mut() {
                    if let Some(idx) = step_index {
                        if let Some(steps) = plan["steps"].as_array_mut() {
                            if idx < steps.len() {
                                steps[idx]["status"] = Value::String(status.to_string());
                                if let Some(n) = notes { steps[idx]["notes"] = Value::String(n.to_string()); }
                                plan["current_step"] = Value::Number(serde_json::Number::from(idx + 1));
                            }
                        }
                    }
                    ok(serde_json::json!({"plan": plan.clone()}))
                } else { err("No active plan. Use action=create first.") }
            }
            "get" => {
                let state = PLAN_STATE.lock().unwrap();
                ok(serde_json::json!({"plans": state.clone()}))
            }
            _ => err(&format!("Unknown action: {action}. Use create/update/get")),
        }
    }
}
pub fn update_plan_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "action":{"type":"string","description":"create/update/get"},
        "title":{"type":"string","description":"Plan title (for create)"},
        "steps":{"type":"array","items":{"type":"object"},"description":"Step objects"},
        "step_index":{"type":"integer","description":"Step index to update"},
        "status":{"type":"string","description":"Step status"},
        "notes":{"type":"string","description":"Step notes"}
    },"required":["action"]})
}

/// `update_todo` 工具处理器（增强版）：持久化 TODO 列表
pub struct UpdateTodoHandler;
#[async_trait]
impl ToolHandler for UpdateTodoHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let action = required_str(&args, "action").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let todo_path = std::env::temp_dir().join("deepwhale_todos.json");
        let mut todos: Vec<serde_json::Value> = std::fs::read_to_string(&todo_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        match action {
            "add" => {
                let content = required_str(&args, "content").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
                let status = optional_str(&args, "status").unwrap_or("pending");
                todos.push(serde_json::json!({"id": uuid::Uuid::new_v4().to_string(), "content": content, "status": status}));
                let _ = std::fs::write(&todo_path, serde_json::to_string_pretty(&todos).unwrap_or_default());
                ok(serde_json::json!({"added": content, "total": todos.len()}))
            }
            "list" => {
                let filter = optional_str(&args, "status");
                let items: Vec<&Value> = todos.iter().filter(|t| {
                    filter.map_or(true, |s| t["status"].as_str() == Some(s))
                }).collect();
                ok(serde_json::json!({"todos": items, "total": items.len(), "all_total": todos.len()}))
            }
            "done" | "complete" => {
                let id = required_str(&args, "id").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
                if let Some(t) = todos.iter_mut().find(|t| t["id"].as_str() == Some(id)) {
                    t["status"] = Value::String("completed".to_string());
                    let _ = std::fs::write(&todo_path, serde_json::to_string_pretty(&todos).unwrap_or_default());
                    ok(serde_json::json!({"completed": id}))
                } else { err("Todo not found") }
            }
            "delete" => {
                let id = required_str(&args, "id").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
                let before = todos.len();
                todos.retain(|t| t["id"].as_str() != Some(id));
                let _ = std::fs::write(&todo_path, serde_json::to_string_pretty(&todos).unwrap_or_default());
                ok(serde_json::json!({"deleted": before - todos.len() != 0, "remaining": todos.len()}))
            }
            _ => err("Unknown action. Use add/list/done/delete"),
        }
    }
}
pub fn update_todo_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "action":{"type":"string","description":"add/list/done/delete"},
        "content":{"type":"string","description":"Todo content (for add)"},
        "status":{"type":"string","description":"Filter or set status"},
        "id":{"type":"string","description":"Todo ID (for done/delete)"}
    },"required":["action"]})
}

// ═══════════════════════════════════════════════════════════════════
// 新增工具处理器：文档/数据处理
// ═══════════════════════════════════════════════════════════════════

/// `validate_data` 工具处理器：验证 JSON/TOML 内容
pub struct ValidateDataHandler;
#[async_trait]
impl ToolHandler for ValidateDataHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let format = optional_str(&args, "format").unwrap_or("json");
        let content = required_str(&args, "content").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let file_path = optional_str(&args, "file_path");

        // If file_path is given, read from file
        let data = if let Some(fp) = file_path {
            match std::fs::read_to_string(fp) {
                Ok(c) => c,
                Err(e) => return err(&format!("Failed to read file: {e}")),
            }
        } else { content.to_string() };

        match format {
            "json" => {
                match serde_json::from_str::<Value>(&data) {
                    Ok(v) => ok(serde_json::json!({"valid": true, "type": "json", "parsed": v})),
                    Err(e) => ok(serde_json::json!({"valid": false, "type": "json", "error": e.to_string()})),
                }
            }
            "toml" => {
                // Try to parse as TOML; use toml::Table mapping type
                #[derive(serde::Deserialize)]
                struct TomlRoot(std::collections::BTreeMap<String, toml::Value>);
                match toml::from_str::<toml::Value>(&data) {
                    Ok(v) => {
                        let json_value = serde_json::to_value(&v).unwrap_or(Value::Null);
                        ok(serde_json::json!({"valid": true, "type": "toml", "parsed": json_value}))
                    }
                    Err(e) => ok(serde_json::json!({"valid": false, "type": "toml", "error": e.to_string()})),
                }
            }
            _ => err("Unsupported format. Use json or toml."),
        }
    }
}
pub fn validate_data_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "format":{"type":"string","description":"json or toml"},
        "content":{"type":"string","description":"Inline content to validate"},
        "file_path":{"type":"string","description":"File path to read instead of inline content"}
    },"required":[]})
}

/// `project_map` 工具处理器：生成项目结构概览
pub struct ProjectMapHandler;
#[async_trait]
impl ToolHandler for ProjectMapHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let _args = parse_args(&inv)?;
        let ws = crate::sandbox::get_workspace();
        if ws.is_none() {
            return err("No workspace set.");
        }
        let ws = ws.unwrap();
        // Use project_context's context pack
        let project_block = crate::project_context::build_full_project_block(&ws);
        let mut tree: Vec<Value> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&ws) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
                let kind = if path.is_dir() { "dir" } else { "file" };
                let size = if path.is_file() { std::fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0) } else { 0 };
                tree.push(serde_json::json!({"name": name, "kind": kind, "size": size}));
            }
        }
        ok(serde_json::json!({"workspace": ws.to_string_lossy().to_string(), "tree": tree, "project_context_available": !project_block.is_empty()}))
    }
}
pub fn project_map_schema() -> Value {
    serde_json::json!({"type":"object","properties":{},"required":[]})
}

/// `run_tests` 工具处理器：运行 cargo test
pub struct RunTestsHandler;
#[async_trait]
impl ToolHandler for RunTestsHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let extra_args = optional_str(&args, "args").unwrap_or("");
        let cwd = optional_str(&args, "cwd").map(|p| crate::sandbox::resolve_path(&PathBuf::from(p)));
        let mut cmd = Command::new("cargo");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.arg("test");
        if !extra_args.is_empty() { cmd.arg(extra_args); }
        if let Some(d) = cwd { cmd.current_dir(d); }
        match cmd.output().await {
            Ok(o) => ok(serde_json::json!({
                "exit_code": o.status.code().unwrap_or(-1),
                "stdout": String::from_utf8_lossy(&o.stdout).to_string(),
                "stderr": String::from_utf8_lossy(&o.stderr).to_string(),
            })),
            Err(e) => err(&format!("cargo test failed: {e}")),
        }
    }
}
pub fn run_tests_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "args":{"type":"string","description":"Extra cargo test arguments"},
        "cwd":{"type":"string","description":"Working directory"}
    },"required":[]})
}

/// `finance_quote` 工具处理器：获取股票报价（Yahoo Finance）
pub struct FinanceQuoteHandler;
#[async_trait]
impl ToolHandler for FinanceQuoteHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let symbol = required_str(&args, "symbol").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0")
            .build().map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string() })?;
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => {
                match r.json::<Value>().await {
                    Ok(data) => ok(serde_json::json!({"symbol": symbol, "data": data})),
                    Err(e) => err(&format!("Failed to parse quote: {e}")),
                }
            }
            Ok(r) => err(&format!("Yahoo Finance API returned {}", r.status())),
            Err(e) => err(&format!("Request failed: {e}")),
        }
    }
}
pub fn finance_quote_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"symbol":{"type":"string","description":"Stock symbol (e.g. AAPL, GOOGL)"}},"required":["symbol"]})
}

/// `pandoc_convert` 工具处理器：文档格式转换
///
/// 优先使用 bundled pandoc（exe 同目录下），回退到系统 PATH。
pub struct PandocConvertHandler;
#[async_trait]
impl ToolHandler for PandocConvertHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let from = optional_str(&args, "from").unwrap_or("markdown");
        let to = required_str(&args, "to").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let content = required_str(&args, "content").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let timeout = optional_str(&args, "timeout_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(15000);
        // Try bundled pandoc exe first, then system PATH
        let pandoc_path = find_bundled_pandoc().unwrap_or_else(|| "pandoc".to_string());
        // Write input to temp file
        let input_file = std::env::temp_dir().join(format!("pandoc_input_{}.tmp", uuid::Uuid::new_v4()));
        let output_file = std::env::temp_dir().join(format!("pandoc_output_{}.tmp", uuid::Uuid::new_v4()));
        let _ = std::fs::write(&input_file, content);
        let mut cmd = Command::new(&pandoc_path);
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.args(["-f", from, "-t", to, "-o", &output_file.to_string_lossy(), &input_file.to_string_lossy()]);
        match tokio::time::timeout(Duration::from_millis(timeout), cmd.output()).await {
            Ok(Ok(o)) if o.status.success() => {
                let result = std::fs::read_to_string(&output_file).unwrap_or_default();
                let _ = std::fs::remove_file(&input_file);
                let _ = std::fs::remove_file(&output_file);
                ok(serde_json::json!({"converted": result, "from": from, "to": to}))
            }
            Ok(Ok(o)) => {
                let _ = std::fs::remove_file(&input_file);
                let _ = std::fs::remove_file(&output_file);
                err(&format!("pandoc failed: {}", String::from_utf8_lossy(&o.stderr)))
            }
            Ok(Err(e)) => {
                let _ = std::fs::remove_file(&input_file);
                err(&format!("pandoc not found or error: {e}"))
            }
            Err(_) => {
                let _ = std::fs::remove_file(&input_file);
                err("pandoc timed out")
            }
        }
    }
}

/// 查找 bundled pandoc 可执行文件路径（exe 同级目录）。
fn find_bundled_pandoc() -> Option<String> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    for rel in &[
        "",
        "binaries/",
        "_resources/",
        "resources/",
        "_resources/binaries/",
        "resources/binaries/",
    ] {
        let candidate = exe_dir.join(rel).join("pandoc.exe");
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}
pub fn pandoc_convert_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "from":{"type":"string","description":"Source format (default: markdown)"},
        "to":{"type":"string","description":"Target format (html, latex, docx, rst, etc.)"},
        "content":{"type":"string","description":"Content to convert"},
        "timeout_ms":{"type":"integer","description":"Timeout in ms (default 15000)"}
    },"required":["to","content"]})
}

/// `js_execution` 工具处理器：执行 JavaScript 代码
pub struct JsExecutionHandler;
#[async_trait]
impl ToolHandler for JsExecutionHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { true }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let code = required_str(&args, "code").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let tmp = std::env::temp_dir().join(format!("dw_js_{}.js", uuid::Uuid::new_v4()));
        let _ = std::fs::write(&tmp, code);
        let mut cmd = Command::new("node");
        crate::sandbox::confine_command(&mut cmd, crate::sandbox::global_sandbox_spec());
        cmd.arg(&tmp);
        match tokio::time::timeout(Duration::from_secs(15), cmd.output()).await {
            Ok(Ok(o)) => {
                let _ = std::fs::remove_file(&tmp);
                ok(serde_json::json!({
                    "stdout": String::from_utf8_lossy(&o.stdout).to_string(),
                    "stderr": String::from_utf8_lossy(&o.stderr).to_string(),
                    "exit_code": o.status.code().unwrap_or(-1)
                }))
            }
            Ok(Err(e)) => { let _ = std::fs::remove_file(&tmp); err(&format!("node execution failed: {e}")) }
            Err(_) => { let _ = std::fs::remove_file(&tmp); err("JavaScript execution timed out (15s)") }
        }
    }
}
pub fn js_execution_schema() -> Value {
    serde_json::json!({"type":"object","properties":{"code":{"type":"string","description":"JavaScript code to execute"}},"required":["code"]})
}

// ═══════════════════════════════════════════════════════════════════
// WebSearch 增强版 + 多后端支持
// ═══════════════════════════════════════════════════════════════════

/// 使用 Bing 搜索（无需 API key，HTML 抓取）
pub async fn web_search_bing(query: &str) -> Result<Vec<Value>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build().map_err(|e| format!("HTTP client: {e}"))?;
    let url = format!("https://www.bing.com/search?q={}", urlencoding_crate::urlencode(query));
    let resp = client.get(&url).send().await.map_err(|e| format!("Bing request failed: {e}"))?;
    let text = resp.text().await.map_err(|e| format!("Read failed: {e}"))?;
    let mut results = Vec::new();
    // Simple HTML parsing for Bing results
    for chunk in text.split("<li class=\"b_algo\"") {
        if chunk.contains("<a href=\"") {
            if let Some(start) = chunk.find("<a href=\"") {
                let after_href = &chunk[start + 9..];
                if let Some(end) = after_href.find('\"') {
                    let link = &after_href[..end];
                    // Extract title
                    let title = chunk.split("<h2>").nth(1)
                        .and_then(|s| s.split("</h2>").next())
                        .unwrap_or("")
                        .replace("<strong>", "").replace("</strong>", "");
                    results.push(serde_json::json!({"title": title, "url": link}));
                }
            }
        }
        if results.len() >= 5 { break; }
    }
    Ok(results)
}

/// URL 编码（用于搜索请求）
mod urlencoding_crate {
    pub fn urlencode(text: &str) -> String {
        text.chars().map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        }).collect()
    }
}

/// `web_search` 增强版处理器（支持多后端）
pub struct EnhancedWebSearchHandler;
#[async_trait]
impl ToolHandler for EnhancedWebSearchHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let q = required_str(&args, "query").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let backend = optional_str(&args, "backend").unwrap_or("auto");
        let results = match backend {
            "bing" => web_search_bing(q).await,
            _ => crate::tools::web_search_simple(q).await, // default DDG
        };
        match results {
            Ok(r) => ok(serde_json::json!({"results": r, "backend": backend})),
            Err(e) => {
                // Fallback to DDG if Bing fails
                if backend != "auto" && backend != "duckduckgo" {
                    if let Ok(r) = crate::tools::web_search_simple(q).await {
                        return ok(serde_json::json!({"results": r, "backend": "duckduckgo_fallback"}));
                    }
                }
                err(&format!("Web search ({backend}): {e}"))
            }
        }
    }
}
pub fn enhanced_web_search_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "query":{"type":"string"},
        "backend":{"type":"string","description":"Search backend: auto, duckduckgo, bing"},
        "max_results":{"type":"integer"}
    },"required":["query"]})
}

/// `fetch_url` 增强版：添加 HTML→Markdown 转换
pub struct EnhancedFetchUrlHandler;
#[async_trait]
impl ToolHandler for EnhancedFetchUrlHandler {
    fn kind(&self) -> ToolKind { ToolKind::Function }
    fn is_mutating(&self) -> bool { false }
    async fn handle(&self, inv: ToolInvocation) -> std::result::Result<ToolOutput, FunctionCallError> {
        let args = parse_args(&inv)?;
        let url = required_str(&args, "url").map_err(|e| FunctionCallError::ExecutionFailed { name: inv.tool_name.clone(), error: e.to_string(), })?;
        let raw = optional_str(&args, "raw").unwrap_or("false") == "true";
        let max_size = optional_str(&args, "max_size").and_then(|s| s.parse::<usize>().ok()).unwrap_or(100_000);
        let timeout = optional_str(&args, "timeout_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(15000);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout))
            .user_agent("Mozilla/5.0 (compatible; DeepWhale/1.0)")
            .build().map_err(|e| FunctionCallError::ExecutionFailed { name: "fetch_url".into(), error: e.to_string() })?;
        match client.get(url).send().await {
            Ok(r) => {
                let status = r.status().as_u16();
                let content_type = r.headers().get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                match r.text().await {
                    Ok(t) => {
                        let raw_size = t.len();
                        let display = if !raw && content_type.contains("text/html") {
                            html2md::parse_html(&t)
                        } else { t };
                        ok(serde_json::json!({
                            "status": status,
                            "content_type": content_type,
                            "body": display,
                            "truncated": raw_size > max_size,
                            "raw_size": raw_size
                        }))
                    }
                    Err(e) => err(&format!("Read failed: {e}")),
                }
            }
            Err(e) => err(&format!("HTTP failed: {e}")),
        }
    }
}
pub fn enhanced_fetch_url_schema() -> Value {
    serde_json::json!({"type":"object","properties":{
        "url":{"type":"string"},
        "raw":{"type":"boolean","description":"Return raw HTML instead of Markdown"},
        "max_size":{"type":"integer","description":"Maximum bytes to fetch"},
        "timeout_ms":{"type":"integer","description":"Timeout in ms (default 15000)"}
    },"required":["url"]})
}

#[cfg(test)]
mod tests {
    /// 工具处理器的单元测试模块。
    ///
    /// 测试内容：
    /// - `test_write_file_creates_file`：验证 write_file 能正确创建并写入文件
    /// - `test_write_file_relative_path`：验证 write_file 能处理嵌套路径
    use super::*;
    use nyamu_tools::ToolCallSource;

    /// 测试 `write_file` 创建新文件的功能：
    /// 写入 "hello world" 到临时文件后，验证文件存在且内容正确。
    #[tokio::test]
    async fn test_write_file_creates_file() {
        let tmp = std::env::temp_dir().join("dw_test_write_file.txt");
        let _ = std::fs::remove_file(&tmp);
        let args = serde_json::json!({"path": tmp.to_string_lossy(), "content": "hello world"});
        let inv = ToolInvocation {
            call_id: "test-1".into(),
            tool_name: "write_file".into(),
            payload: ToolPayload::Function { arguments: args.to_string() },
            source: ToolCallSource::Direct,
        };
        let handler = WriteFileHandler;
        let result = handler.handle(inv).await;
        assert!(result.is_ok(), "handler should return Ok, got {:?}", result);
        let output = result.unwrap();
        match &output {
            ToolOutput::Function { body, success } => {
                assert!(*success, "write should succeed, body: {:?}", body);
            }
            _ => panic!("unexpected output: {:?}", output),
        }
        assert!(tmp.exists(), "File should exist after write_file: {}", tmp.display());
        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert_eq!(content, "hello world");
        let _ = std::fs::remove_file(&tmp);
    }

    /// 测试 `write_file` 处理相对路径和嵌套目录的功能：
    /// 写入文件到 `nested/sub/` 子目录中，验证自动创建父目录逻辑正常。
    #[tokio::test]
    async fn test_write_file_relative_path() {
        let test_root = std::env::temp_dir().join("dw_test_rel");
        let _ = std::fs::create_dir_all(&test_root);
        let file_path = test_root.join("nested/sub/file.txt");
        let file_path_str = file_path.to_string_lossy().to_string();
        let args = serde_json::json!({"path": file_path_str, "content": "relative test"});
        let inv = ToolInvocation {
            call_id: "test-2".into(),
            tool_name: "write_file".into(),
            payload: ToolPayload::Function { arguments: args.to_string() },
            source: ToolCallSource::Direct,
        };
        let handler = WriteFileHandler;
        let result = handler.handle(inv).await;
        assert!(result.is_ok(), "handler should return Ok, got {:?}", result);
        assert!(file_path.exists(), "File should exist: {}", file_path.display());
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        assert_eq!(content, "relative test");
        let _ = std::fs::remove_dir_all(&test_root);
    }
}