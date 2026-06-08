//! 工具注册表模块。
//!
//! 本模块负责 DeepWhale 系统中所有工具的注册、调度以及与外部 API 格式的转换。
//!
//! # 架构
//! - 每个工具由一个 `ToolSpec`（工具规格定义）和一个实现了 `ToolHandler` trait 的处理器组成，
//!   统一注册在 `ToolRegistry` 中。
//! - `build_registry()` 注册所有内置工具（共 13 个），包括文件读写、搜索、Web 访问、待办事项等。
//! - `build_api_tools_for_mode()` 根据当前模式（如 Plan 模式）过滤工具列表，生成符合
//!   DeepSeek/OpenAI 函数调用 API 格式的 JSON 定义。
//! - `dispatch_tool_call()` 和 `dispatch_tool_call_direct()` 负责处理来自 API 或 CLI 的
//!   工具调用请求，将字符串参数解析后转发给对应的处理器。
//! - 工具调用的完整生命周期由 `engine.rs` 中的 agent 循环协调。
//!
//! # 安全机制
//! - Plan 模式下会屏蔽写操作工具（write_file、edit_file 等），防止意外修改。
//! - 调度层面也有硬编码的 Plan 模式检查作为兜底保护。
mod handlers;

use std::sync::Arc;
use anyhow::Result;
use nyamu_tools::{ToolHandler, ToolRegistry, ToolSpec};
use serde_json::Value;

pub use handlers::{get_all_plans, get_latest_plan};

/// 构建包含所有内置工具的完整工具注册表。
///
/// 内部使用 `register!` 宏简化注册流程，为每个工具指定：
/// - 名称（如 `"read_file"`）
/// - JSON Schema 输入定义（来自 handlers 模块的 `xxx_schema()` 函数）
/// - 处理器实例（实现了 `ToolHandler` 的具体结构体）
///
/// # 已注册的工具（共 13 个）
/// - **文件操作**: read_file, write_file, edit_file, apply_patch
/// - **命令执行**: exec_shell
/// - **搜索**: grep_files, file_search, list_dir
/// - **网络**: web_search, fetch_url
/// - **待办事项**: todo_write, todo_list
/// - **子代理**: sub_agent
///
/// # 返回值
/// 返回构建好的 `ToolRegistry` 实例。若注册过程出错则返回错误。
pub fn build_registry() -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();

    macro_rules! register {
        ($name:expr, $spec_json:expr, $handler:expr) => {{
            let spec = ToolSpec {
                name: $name.to_string(),
                input_schema: $spec_json.clone(),
                output_schema: serde_json::json!({"type": "string"}),
                supports_parallel_tool_calls: true,
                timeout_ms: Some(30000),
            };
            registry.register(spec, $handler as Arc<dyn ToolHandler>)?;
        }};
    }

    register!("read_file",     handlers::read_file_schema(),     Arc::new(handlers::ReadFileHandler));
    register!("write_file",    handlers::write_file_schema(),    Arc::new(handlers::WriteFileHandler));
    register!("edit_file",     handlers::edit_file_schema(),     Arc::new(handlers::EditFileHandler));
    register!("apply_patch",   handlers::apply_patch_schema(),   Arc::new(handlers::ApplyPatchHandler));
    register!("exec_shell",    handlers::exec_shell_schema(),    Arc::new(handlers::ExecShellHandler));
    register!("grep_files",    handlers::grep_files_schema(),    Arc::new(handlers::GrepFilesHandler));
    register!("file_search",   handlers::file_search_schema(),   Arc::new(handlers::FileSearchHandler));
    register!("list_dir",      handlers::list_dir_schema(),      Arc::new(handlers::ListDirHandler));
    register!("web_search",    handlers::web_search_schema(),    Arc::new(handlers::WebSearchHandler));
    register!("fetch_url",     handlers::fetch_url_schema(),     Arc::new(handlers::FetchUrlHandler));
    register!("todo_write",    handlers::todo_write_schema(),    Arc::new(handlers::TodoWriteHandler));
    register!("todo_list",     handlers::todo_list_schema(),     Arc::new(handlers::TodoListHandler));
    register!("sub_agent",     handlers::sub_agent_schema(),     Arc::new(handlers::SubAgentHandler));

    // ── 新工具注册 ─────────────────────────────────────────
    register!("git_status",    handlers::git_status_schema(),    Arc::new(handlers::GitStatusHandler));
    register!("git_diff",      handlers::git_diff_schema(),      Arc::new(handlers::GitDiffHandler));
    register!("git_log",       handlers::git_log_schema(),       Arc::new(handlers::GitLogHandler));
    register!("git_show",      handlers::git_show_schema(),      Arc::new(handlers::GitShowHandler));
    register!("git_blame",     handlers::git_blame_schema(),     Arc::new(handlers::GitBlameHandler));
    register!("github_issue",   handlers::github_issue_schema(), Arc::new(handlers::GitHubIssueHandler));
    register!("github_pr",     handlers::github_pr_schema(),     Arc::new(handlers::GitHubPRHandler));
    register!("github_comment", handlers::github_comment_schema(), Arc::new(handlers::GitHubCommentHandler));
    register!("notify",        handlers::notify_schema(),        Arc::new(handlers::NotifyHandler));
    register!("remember",      handlers::remember_schema(),      Arc::new(handlers::RememberHandler));
    register!("diagnostics",   handlers::diagnostics_schema(),   Arc::new(handlers::DiagnosticsHandler));
    register!("revert_turn",   handlers::revert_turn_schema(),   Arc::new(handlers::RevertTurnHandler));
    register!("update_plan",   handlers::update_plan_schema(),   Arc::new(handlers::UpdatePlanHandler));
    register!("update_todo",   handlers::update_todo_schema(),   Arc::new(handlers::UpdateTodoHandler));
    register!("validate_data", handlers::validate_data_schema(), Arc::new(handlers::ValidateDataHandler));
    register!("project_map",   handlers::project_map_schema(),   Arc::new(handlers::ProjectMapHandler));
    register!("run_tests",     handlers::run_tests_schema(),     Arc::new(handlers::RunTestsHandler));
    register!("finance_quote", handlers::finance_quote_schema(), Arc::new(handlers::FinanceQuoteHandler));
    register!("pandoc_convert",handlers::pandoc_convert_schema(),Arc::new(handlers::PandocConvertHandler));
    register!("js_execution",  handlers::js_execution_schema(),  Arc::new(handlers::JsExecutionHandler));

    // Enhanced versions replace originals
    register!("web_search",     handlers::enhanced_web_search_schema(), Arc::new(handlers::EnhancedWebSearchHandler));
    register!("fetch_url",      handlers::enhanced_fetch_url_schema(),  Arc::new(handlers::EnhancedFetchUrlHandler));

    Ok(registry)
}

/// 在 Plan 模式下被屏蔽的写操作工具列表。
///
/// Plan 模式是一种只读模式，仅允许查看和分析代码，禁止任何修改操作。
/// 包含在此列表中的工具在 Plan 模式下会从工具列表中移除，并在调度时被拦截。
/// 这是防止 AI 在规划阶段意外修改文件的重要安全机制。
const WRITE_TOOLS: &[&str] = &["write_file", "edit_file", "apply_patch", "exec_shell", "todo_write", "github_comment", "update_plan", "update_todo", "revert_turn", "run_tests", "js_execution"];

/// 将 `ToolSpec` 转换为 DeepSeek / OpenAI 兼容的函数定义格式。
///
/// 生成的结构符合 OpenAI 函数调用 API 规范：
/// ```json
/// {
///   "type": "function",
///   "function": {
///     "name": "...",
///     "description": "...",
///     "parameters": { ... }
///   }
/// }
/// ```
/// 此类 API 格式可被 DeepSeek、OpenAI 等大模型直接识别。
pub fn spec_to_api_tool(spec: &ToolSpec) -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": spec.name,
            "description": spec_desc(&spec.name),
            "parameters": spec.input_schema,
        }
    })
}

/// 构建完整的工具 API 数组（无模式过滤，全部工具返回）。
///
/// 这是 `build_api_tools_for_mode` 的简化版本，直接传递 `None` 作为模式参数，
/// 因此会返回注册表中的所有工具，不做任何屏蔽。
/// 通常用于非 Plan 模式的普通对话场景。
pub fn build_api_tools(registry: &ToolRegistry) -> Vec<Value> {
    build_api_tools_for_mode(registry, None)
}

/// 根据当前模式构建工具 API 数组，支持 Plan 模式过滤。
///
/// 当模式为 `"plan"` 时，会自动从返回列表中移除写操作工具（`WRITE_TOOLS`），
/// 确保规划阶段的大模型只能执行只读操作，无法修改文件系统。
///
/// # 参数
/// - `registry`: 工具注册表引用
/// - `mode`: 当前模式，传入 `Some("plan")` 启用过滤，`None` 或其它值不过滤
///
/// # 返回值
/// 返回一个 Vec<Value>，每个元素是对应工具的 DeepSeek/OpenAI API 格式定义。
pub fn build_api_tools_for_mode(registry: &ToolRegistry, mode: Option<&str>) -> Vec<Value> {
    let is_plan = mode.map(|m| m.eq_ignore_ascii_case("plan")).unwrap_or(false);
    registry.list_specs().iter().filter(|cfg| {
        !(is_plan && WRITE_TOOLS.contains(&cfg.spec.name.as_str()))
    }).map(|cfg| spec_to_api_tool(&cfg.spec)).collect()
}

/// 检查指定工具在当前模式下是否允许调用。
///
/// 这是 Plan 模式安全机制的第二道防线（第一道是 `build_api_tools_for_mode` 的过滤）。
/// 即使工具列表已过滤，此处仍然在调度入口进行二次检查，防止绕过。
///
/// # 参数
/// - `name`: 工具名称
/// - `mode`: 当前模式，`None` 视为普通 agent 模式
///
/// # 返回值
/// - `Ok(())`: 允许调用
/// - `Err(String)`: 拒绝调用，返回错误消息（如 `"'write_file' not allowed in Plan mode"`）
pub fn check_mode_tool_allowed(name: &str, mode: Option<&str>) -> Result<(), String> {
    let m = mode.unwrap_or("agent");
    if m.eq_ignore_ascii_case("plan") && WRITE_TOOLS.contains(&name) {
        return Err(format!("'{name}' not allowed in Plan mode"));
    }
    Ok(())
}

/// 从 API 响应中调度单个工具调用。
///
/// 接收来自大模型（如 DeepSeek）函数调用返回的工具名称和 JSON 参数，
/// 将其封装为 `ToolPayload::Function`，提交给 `ToolRegistry` 的 dispatch 方法执行。
///
/// # Plan 模式安全兜底
/// 在调度层面硬编码了 Plan 模式检查：如果环境变量 `NYAMU_MODE` 设置为 `"plan"`，
/// 即使工具列表过滤被绕过，此处仍然会拦截写操作工具调用并返回错误。
///
/// # 参数
/// - `registry`: 工具注册表引用
/// - `name`: 要调用的工具名称
/// - `arguments`: JSON 格式的参数字符串
///
/// # 返回值
/// - `Ok(String)`: 工具执行成功，返回格式化的 JSON 结果
/// - `Err(anyhow::Error)`: 执行失败，返回错误信息
pub async fn dispatch_tool_call(
    registry: &ToolRegistry,
    name: &str,
    arguments: &str,
) -> Result<String> {
    // Hard Plan mode check at dispatch level (safety net)
    if WRITE_TOOLS.contains(&name) {
        // Check if running in Plan mode by checking env or a static flag
        if std::env::var("NYAMU_MODE").ok().as_deref() == Some("plan") {
            return Err(anyhow::anyhow!("Blocked by Plan mode: {} is read-only", name));
        }
    }
    let payload = nyamu_protocol::ToolPayload::Function {
        arguments: arguments.to_string(),
    };
    let call = nyamu_tools::ToolCall {
        name: name.to_string(),
        payload,
        source: nyamu_tools::ToolCallSource::Direct,
        raw_tool_call_id: None,
    };
    let result = registry
        .dispatch(call, true)
        .await
        .map_err(|e| anyhow::anyhow!("Tool error: {:?}", e))?;
    match result {
        nyamu_protocol::ToolOutput::Function { body, success } => {
            let content = body
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                .unwrap_or_default();
            if success {
                Ok(content)
            } else {
                Err(anyhow::anyhow!("Tool failed: {}", content))
            }
        }
        nyamu_protocol::ToolOutput::Mcp { result: val } => {
            Ok(serde_json::to_string_pretty(&val).unwrap_or_default())
        }
    }
}

/// 直接执行工具调用（绕过 API 协议封装）。
///
/// 此函数用于 CLI exec agent 模式，它不经过 `ToolRegistry` 的 dispatch 流程，
/// 而是直接根据工具名称匹配对应的处理器，手动构造 `ToolInvocation` 后执行。
/// 这种方式更轻量，适用于不需要完整注册表体系的场景。
///
/// # 参数
/// - `name`: 工具名称，必须是已注册的 13 个工具之一
/// - `arguments`: JSON 格式的参数字符串
///
/// # 返回值
/// - `Ok(String)`: 执行成功，返回格式化的 JSON 结果
/// - `Err(String)`: 执行失败，返回错误消息。如果工具名称不存在返回 `"Unknown tool: {name}"`。
///
/// # 支持的工具有
/// read_file, write_file, edit_file, apply_patch, exec_shell, grep_files,
/// file_search, list_dir, web_search, fetch_url, todo_write, todo_list, sub_agent
pub async fn dispatch_tool_call_direct(name: &str, arguments: &str) -> Result<String, String> {
    use std::sync::Arc;
    let handler = match name {
        "read_file" => Arc::new(handlers::ReadFileHandler) as Arc<dyn ToolHandler>,
        "write_file" => Arc::new(handlers::WriteFileHandler) as Arc<dyn ToolHandler>,
        "edit_file" => Arc::new(handlers::EditFileHandler) as Arc<dyn ToolHandler>,
        "apply_patch" => Arc::new(handlers::ApplyPatchHandler) as Arc<dyn ToolHandler>,
        "exec_shell" => Arc::new(handlers::ExecShellHandler) as Arc<dyn ToolHandler>,
        "grep_files" => Arc::new(handlers::GrepFilesHandler) as Arc<dyn ToolHandler>,
        "file_search" => Arc::new(handlers::FileSearchHandler) as Arc<dyn ToolHandler>,
        "list_dir" => Arc::new(handlers::ListDirHandler) as Arc<dyn ToolHandler>,
        "web_search" => Arc::new(handlers::EnhancedWebSearchHandler) as Arc<dyn ToolHandler>,
        "fetch_url" => Arc::new(handlers::EnhancedFetchUrlHandler) as Arc<dyn ToolHandler>,
        "todo_write" => Arc::new(handlers::TodoWriteHandler) as Arc<dyn ToolHandler>,
        "todo_list" => Arc::new(handlers::TodoListHandler) as Arc<dyn ToolHandler>,
        "sub_agent" => Arc::new(handlers::SubAgentHandler) as Arc<dyn ToolHandler>,
        // New tools
        "git_status" => Arc::new(handlers::GitStatusHandler) as Arc<dyn ToolHandler>,
        "git_diff" => Arc::new(handlers::GitDiffHandler) as Arc<dyn ToolHandler>,
        "git_log" => Arc::new(handlers::GitLogHandler) as Arc<dyn ToolHandler>,
        "git_show" => Arc::new(handlers::GitShowHandler) as Arc<dyn ToolHandler>,
        "git_blame" => Arc::new(handlers::GitBlameHandler) as Arc<dyn ToolHandler>,
        "github_issue" => Arc::new(handlers::GitHubIssueHandler) as Arc<dyn ToolHandler>,
        "github_pr" => Arc::new(handlers::GitHubPRHandler) as Arc<dyn ToolHandler>,
        "github_comment" => Arc::new(handlers::GitHubCommentHandler) as Arc<dyn ToolHandler>,
        "notify" => Arc::new(handlers::NotifyHandler) as Arc<dyn ToolHandler>,
        "remember" => Arc::new(handlers::RememberHandler) as Arc<dyn ToolHandler>,
        "diagnostics" => Arc::new(handlers::DiagnosticsHandler) as Arc<dyn ToolHandler>,
        "revert_turn" => Arc::new(handlers::RevertTurnHandler) as Arc<dyn ToolHandler>,
        "update_plan" => Arc::new(handlers::UpdatePlanHandler) as Arc<dyn ToolHandler>,
        "update_todo" => Arc::new(handlers::UpdateTodoHandler) as Arc<dyn ToolHandler>,
        "validate_data" => Arc::new(handlers::ValidateDataHandler) as Arc<dyn ToolHandler>,
        "project_map" => Arc::new(handlers::ProjectMapHandler) as Arc<dyn ToolHandler>,
        "run_tests" => Arc::new(handlers::RunTestsHandler) as Arc<dyn ToolHandler>,
        "finance_quote" => Arc::new(handlers::FinanceQuoteHandler) as Arc<dyn ToolHandler>,
        "pandoc_convert" => Arc::new(handlers::PandocConvertHandler) as Arc<dyn ToolHandler>,
        "js_execution" => Arc::new(handlers::JsExecutionHandler) as Arc<dyn ToolHandler>,
        _ => return Err(format!("Unknown tool: {name}")),
    };
    use nyamu_protocol::ToolPayload;
    use nyamu_tools::{ToolInvocation, ToolCallSource};
    let inv = ToolInvocation {
        call_id: uuid::Uuid::new_v4().to_string(),
        tool_name: name.to_string(),
        payload: ToolPayload::Function { arguments: arguments.to_string() },
        source: ToolCallSource::Direct,
    };
    match handler.handle(inv).await {
        Ok(output) => {
            match output {
                nyamu_protocol::ToolOutput::Function { body, success } => {
                    let text = body.map(|v| serde_json::to_string_pretty(&v).unwrap_or_default()).unwrap_or_default();
                    if success { Ok(text) } else { Err(text) }
                }
                nyamu_protocol::ToolOutput::Mcp { result } => {
                    Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
                }
            }
        }
        Err(e) => Err(format!("Tool error: {:?}", e)),
    }
}

/// 使用 DuckDuckGo Lite 的轻量级网页搜索实现。
///
/// 发送 HTTP GET 请求到 DuckDuckGo Lite 版（`https://lite.duckduckgo.com/lite/`），
/// 解析返回的 HTML 页面，提取搜索结果中的标题和 URL 链接。
///
/// 这是一个无需 API Key 的免费搜索方案，适用于简单的网页搜索需求。
/// 由于解析的是 HTML 而非 JSON API，搜索质量可能不如专业搜索服务。
///
/// # 参数
/// - `query`: 搜索关键词
///
/// # 返回值
/// 返回最多 5 条搜索结果，每条包含 `title`（标题）和 `url`（链接）字段。
///
/// # 错误处理
/// 如果 HTTP 请求失败或客户端构建失败，返回对应的错误消息字符串。
pub async fn web_search_simple(query: &str) -> Result<Vec<serde_json::Value>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; DeepWhale/1.0)")
        .build().map_err(|e| format!("Failed to build HTTP client: {e}"))?;
    let url = format!("https://lite.duckduckgo.com/lite/?q={}", urlencoding(&query));
    let resp = client.get(&url).send().await.map_err(|e| format!("Request failed: {e}"))?;
    let text = resp.text().await.map_err(|e| format!("Failed to read response: {e}"))?;
    let mut results = Vec::new();
    // Parse DDG lite HTML: find result links
    let mut in_result = false;
    let mut current_link = String::new();
    let mut current_title = String::new();
    for line in text.lines() {
        if line.contains("class=\"result-link\"") { in_result = true; current_link.clear(); current_title.clear(); }
        if in_result {
            if let Some(href) = extract_href(line) { current_link = href.to_string(); }
            if let Some(title) = extract_title(line) { current_title = title.to_string(); }
            if line.contains("</a>") && !current_link.is_empty() {
                results.push(serde_json::json!({"title": current_title, "url": current_link}));
                in_result = false;
            }
        }
        if results.len() >= 5 { break; }
    }
    Ok(results)
}

/// 对文本进行 URL 编码（百分比编码）。
///
/// 将特殊字符转换为 `%XX` 格式，空格转换为 `+` 号，
/// 符合 application/x-www-form-urlencoded 规范。
/// 字母、数字、`-`、`_`、`.`、`~` 等安全字符保持原样。
///
/// 这是 DuckDuckGo Lite 搜索请求的必要预处理步骤。
fn urlencoding(text: &str) -> String {
    text.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        _ => format!("%{:02X}", c as u32),
    }).collect()
}

/// 从 HTML 行中提取 `href` 属性值。
///
/// 在 DuckDuckGo Lite 的 HTML 结果中查找形如 `href="..."` 的模式，
/// 提取引号内的 URL 链接。如果未找到则返回 `None`。
fn extract_href(line: &str) -> Option<&str> {
    let start = line.find("href=\"")?;
    let after = &line[start + 6..];
    let end = after.find('\"')?;
    Some(&after[..end])
}

/// 从 HTML 行中提取 `>` 和 `<` 之间的文本内容（标题）。
///
/// 用于解析 DuckDuckGo Lite HTML 结果中 `<a>` 标签内的标题文本。
/// 如果提取到的文本为空则返回 `None`。
fn extract_title(line: &str) -> Option<&str> {
    let start = line.find('>')?;
    let after = &line[start + 1..];
    let end = after.find('<')?;
    let t = after[..end].trim();
    if t.is_empty() { None } else { Some(t) }
}

/// 为已知工具名称生成人类可读的描述文本。
///
/// 这些描述会作为 OpenAI/DeepSeek API 函数定义中的 `description` 字段，
/// 帮助大模型理解每个工具的功能和用途，从而做出更准确的选择。
///
/// # 返回值
/// 返回静态字符串引用。对于未知工具名称，返回默认描述 `"A tool"`。
fn spec_desc(name: &str) -> &'static str {
    match name {
        "read_file" => "Read the contents of a file",
        "write_file" => "Create or overwrite a file",
        "edit_file" => "Edit an existing file via search-and-replace",
        "apply_patch" => "Apply a structured diff patch to a file",
        "exec_shell" => "Execute a shell command",
        "grep_files" => "Search file contents with regex patterns",
        "file_search" => "Find files by name",
        "list_dir" => "List directory contents",
        "web_search" => "Search the web using DuckDuckGo or Bing",
        "fetch_url" => "Fetch a URL and return its content as Markdown",
        "todo_write" => "Write a todo item",
        "todo_list" => "List todo items",
        "sub_agent" => "Spawn a child agent to answer a question independently",
        "git_status" => "Show working tree git status",
        "git_diff" => "Show git diff (unstaged or staged changes)",
        "git_log" => "Show recent git commit history",
        "git_show" => "Show detailed commit information",
        "git_blame" => "Show file line annotations with blame info",
        "github_issue" => "View GitHub issue details via gh CLI",
        "github_pr" => "View GitHub pull request details via gh CLI",
        "github_comment" => "Comment on a GitHub issue or PR",
        "notify" => "Send a desktop notification",
        "remember" => "Record a persistent memory note",
        "diagnostics" => "Show workspace and tool diagnostics",
        "revert_turn" => "Revert workspace to a previous snapshot",
        "update_plan" => "Create, update, or view session plan",
        "update_todo" => "Manage persistent todo list",
        "validate_data" => "Validate JSON or TOML content",
        "project_map" => "Show project structure overview",
        "run_tests" => "Run cargo test with optional arguments",
        "finance_quote" => "Get stock/financial quote from Yahoo Finance",
        "pandoc_convert" => "Convert document formats via pandoc",
        "js_execution" => "Execute JavaScript code in Node.js",
        _ => "A tool",
    }
}
