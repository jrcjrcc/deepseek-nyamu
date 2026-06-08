//! DeepWhale 核心引擎 —— Agent 循环与 LLM 交互
//!
//! `DeepWhaleEngine` 是系统的核心，负责：
//! 1. 管理 LLM 对话上下文（conversation_history）
//! 2. 流式通信：通过 Provider trait 发送 ChatRequest，消费 StreamEvent
//! 3. 工具调度：将 LLM 请求的工具调用派发到 ToolRegistry
//! 4. 模式执行：Agent（全部工具）/ Plan（只读）/ Yolo（自动审批）
//! 5. 项目上下文注入（workspace-aware 时自动注入项目结构和 LSP 诊断）
//!
//! Agent 循环（submit_message_with_mode）：
//!   发送消息 → LLM 流式回复 → 如果是 tool_calls → 执行工具 → 继续循环
//!   如果是 stop → 结束对话
use std::path::PathBuf;
use anyhow::{Result, Context};
use tauri::{AppHandle, Emitter};
use crate::sandbox::{SandboxSpec, sandbox_available};
use crate::lsp::{LspManager, LspConfig, render_blocks};
use crate::snapshot::SnapshotManager;
use crate::tools;
use nyamu_tools::ToolRegistry;
use serde_json::Value;
use nyamu_provider::{
    Provider, ProviderRegistry, StreamEvent,
    ChatRequest, ChatMessage, ApiToolCall,
};
use nyamu_execpolicy::{ExecPolicyEngine, ToolAskRule};

/// DeepWhale 核心引擎
///
/// 管理完整的 AI 对话生命周期，包括 LLM 通信、工具调度、
/// 模式执行、项目上下文注入和持久化。
pub struct DeepWhaleEngine {
    pub app_handle: AppHandle,                        // Tauri 应用句柄（用于 emit 事件）
    pub config_store: Option<nyamu_config::ConfigStore>, // 配置存储（API 密钥、提供商等）
    pub sessions: Vec<SessionInfo>,                   // 会话列表（内存中）
    pub active_session_id: Option<String>,            // 当前活跃会话 ID
    pub sandbox_spec: SandboxSpec,                    // 沙盒执行策略
    pub tool_registry: ToolRegistry,                  // 工具注册表（13 个内置工具）
    pub conversation_history: Vec<ChatMessage>,        // 当前会话的对话历史
    pub total_input_tokens: u64,                      // 累计输入 Token 数
    pub total_output_tokens: u64,                     // 累计输出 Token 数
    pub total_cost: f64,                              // 累计费用（美元）
    pub last_cache_hit: u64,                          // 最近一次调用的缓存命中 Token
    pub last_cache_miss: u64,                         // 最近一次调用的缓存未命中 Token
    pub provider_registry: Option<ProviderRegistry>,   // LLM 提供商注册表
    pub is_first_call: bool,                          // 是否首次调用（用于自动路由的首次判断）
    pub workspace: Option<PathBuf>,                   // 工作区目录（启用项目上下文时设置）
    pub lsp_manager: Option<LspManager>,               // LSP 诊断管理器
    pub snapshot_manager: Option<SnapshotManager>,     // 文件系统快照管理器
    pub turn_counter: u64,                            // 当前会话的对话轮次计数
    pub exec_policy: Option<ExecPolicyEngine>,          // 执行策略引擎
}

impl DeepWhaleEngine {
    /// 计算缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.last_cache_hit + self.last_cache_miss;
        if total == 0 { 0.0 } else { self.last_cache_hit as f64 / total as f64 }
    }

    /// 发送桌面通知
    pub fn send_notification(&self, title: &str, message: &str) {
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(message)
            .appname("DeepWhale")
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show();
    }
}

/// 会话元信息（与前端 SessionInfo 对应）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub message_count: u32,
}

/// 单条消息（与前端 Message 对应）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,           // "user" | "assistant" | "tool_call"
    pub content: String,
    pub timestamp: String,
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// 工具调用信息（与前端 ToolCall 对应）
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub input: String,
    pub output: Option<String>,
    pub status: String,  // "running" | "completed" | "failed"
}

impl DeepWhaleEngine {
    /// 创建新的引擎实例
    ///
    /// 初始化流程：
    /// 1. 加载配置（~/.deepwhale/config.toml）
    /// 2. 构建工具注册表
    /// 3. 初始化沙盒策略
    /// 4. 创建 ProviderRegistry
    pub fn new(app_handle: AppHandle) -> Self {
        let config_store = nyamu_config::ConfigStore::load(None).ok();
        let tool_registry = tools::build_registry().unwrap_or_default();
        let sandbox_spec = SandboxSpec {
            mode: if sandbox_available() { "enforce".into() } else { String::new() },
            write_roots: Vec::new(), network: true,
        };
        crate::sandbox::init_global_sandbox_spec(sandbox_spec.clone());
        // Build provider registry from config (providers are created on-demand in submit_message_with_mode)
        let provider_registry = config_store.as_ref().map(|_cs| {
            ProviderRegistry::new()
        });

        Self {
            app_handle, config_store, sessions: Vec::new(),
            active_session_id: None, sandbox_spec, tool_registry,
            conversation_history: Vec::new(),
            total_input_tokens: 0, total_output_tokens: 0, total_cost: 0.0,
            last_cache_hit: 0, last_cache_miss: 0,
            provider_registry, lsp_manager: None,
            snapshot_manager: None,
            turn_counter: 0,

            is_first_call: true,
            workspace: None,
            exec_policy: Some(ExecPolicyEngine::new(Vec::new(), Vec::new())),
        }
    }

    pub async fn submit_message(&mut self, _session_id: &str, content: &str) -> Result<String> {
        self.submit_message_with_mode(_session_id, content, None).await
    }

    pub async fn submit_message_with_mode(&mut self, _session_id: &str, content: &str, mode: Option<&str>) -> Result<String> {
        let config_store = self.config_store.as_ref()
            .context("No config loaded. Set DEEPSEEK_API_KEY env var.")?;

        // Resolve runtime options
        let secrets = nyamu_secrets::Secrets::file_backed();
        let resolved = config_store.config.resolve_runtime_options_with_secrets(
            &nyamu_config::CliRuntimeOverrides { provider: None, model: None,
                api_key: None, base_url: None, auth_mode: None, output_mode: None,
                log_level: None, telemetry: None, approval_policy: None, sandbox_mode: None, yolo: None,
            }, &secrets,
        );

        let raw_model = resolved.model.clone();
        let model = if raw_model.is_empty() || raw_model == "deepseek-chat" {
            "deepseek-v4-flash".to_string()
        } else {
            raw_model
        };

        // Auto-route when model is "auto"
        let model = if model == "auto" || model == "auto_model" {
            let api_key = resolved.api_key.as_deref().unwrap_or("");
            let base_url = resolved.base_url.as_str();
            let latest_request = content;
            let recent_context = self.conversation_history.iter()
                .filter_map(|m| m.content.as_deref())
                .last()
                .unwrap_or("");
            let selection = crate::auto_router::resolve_auto_route(
                api_key, base_url, latest_request, recent_context, false,
            ).await;
            if !self.is_first_call {
                let _ = self.app_handle.emit("model:selected", serde_json::json!({
                    "model": selection.model,
                    "source": format!("{:?}", selection.source),
                }));
            }
            self.is_first_call = false;
            selection.model
        } else {
            model
        };

        let reasoning_effort = std::env::var("DEEPSEEK_REASONING_EFFORT")
            .ok()
            .filter(|v| matches!(v.as_str(), "off"|"low"|"medium"|"high"|"max"))
            .or_else(|| Some("max".to_string()));

        // Set NYAMU_MODE env var so dispatch_tool_call can enforce mode
        if let Some(m) = mode {
            unsafe { std::env::set_var("NYAMU_MODE", m); }
        }

        self.conversation_history.push(ChatMessage {
            role: "user".into(), content: Some(content.to_string()),
            tool_calls: None, tool_call_id: None,
        });

        // Filter tools by mode
        let tools_arr = tools::build_api_tools_for_mode(&self.tool_registry, mode);
        let mut final_text = String::new();

        // Initialize LSP manager if workspace is available
        if self.lsp_manager.is_none() {
            if let Some(ref ws) = self.workspace {
                self.lsp_manager = Some(LspManager::new(LspConfig::default(), ws.clone()));
            }
        }
        // Initialize snapshot manager if workspace is available
        if self.snapshot_manager.is_none() {
            if self.workspace.is_some() {
                self.snapshot_manager = Some(SnapshotManager::new(true));
            }
        }

        loop {
            let mut system_prompt = crate::prompts::build_system_prompt_for_mode(None, mode);
            // Append project context if workspace is set
            if let Some(ref ws) = self.workspace {
                // Pre-turn snapshot
                if let Some(ref sm) = self.snapshot_manager {
                    sm.pre_turn_snapshot(ws, self.turn_counter).await;
                }
                let project_block = crate::project_context::build_full_project_block(ws);
                if !project_block.is_empty() {
                    system_prompt.push_str("\n\n");
                    system_prompt.push_str(&project_block);
                }
            }
            // Append memory if enabled
            let memory_path = crate::memory::resolve_global_memory_path(None);
            let memory_block = crate::memory::build_layered_memory_block(&memory_path, None, None);
            if !memory_block.is_empty() {
                system_prompt.push_str("\n\n");
                system_prompt.push_str(&memory_block);
            }
            let mut messages = Vec::with_capacity(self.conversation_history.len() + 1);
            messages.push(ChatMessage {
                role: "system".into(), content: Some(system_prompt),
                tool_calls: None, tool_call_id: None,
            });
            messages.extend(self.conversation_history.clone());

            // Determine which provider to use based on resolved config
            let provider_kind = resolved.provider;
            let provider_registry = self.provider_registry.get_or_insert_with(ProviderRegistry::new);

            // Get the appropriate provider; fall back to creating one from resolved options
            let inner_provider = match provider_registry.get(provider_kind).await {
                Some(p) => p,
                None => Provider::from_options(provider_kind, resolved.clone()),
            };

            let chat_request = ChatRequest {
                model: model.clone(),
                messages,
                system: None,
                tools: Some(tools_arr.clone()),
                tool_choice: "auto".into(),
                stream: true,
                max_tokens: 16384,
                reasoning_effort: reasoning_effort.clone(),
                temperature: None,
                top_p: None,
                stop: None,
                extra: std::collections::HashMap::new(),
            };

            // Stream the response
            let mut stream = inner_provider.stream_chat(&chat_request).await
                .context("Failed to start chat stream")?;

            use futures::StreamExt;
            let mut full_content = String::new();
            let mut full_reasoning = String::new();
            let mut has_tool_calls = false;
            let mut finish_reason = String::new();
            let mut tool_calls_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

            while let Some(event) = stream.next().await {
                match event? {
                    StreamEvent::Chunk { content, reasoning } => {
                        if !content.is_empty() {
                            full_content.push_str(&content);
                            let _ = self.app_handle.emit("token", serde_json::json!({
                                "token": content, "full": full_content,
                            }));
                        }
                        if let Some(r) = reasoning {
                            full_reasoning.push_str(&r);
                            let _ = self.app_handle.emit("reasoning", serde_json::json!({
                                "token": r, "full": full_reasoning,
                            }));
                        }
                    }
                    StreamEvent::ToolCallDelta { index, id, name, arguments_chunk } => {
                        has_tool_calls = true;
                        let key = format!("tc_{}", index);
                        let entry = tool_calls_map.entry(key).or_insert_with(|| {
                            serde_json::json!({
                                "id": "", "type": "function",
                                "function": {"name": "", "arguments": ""}
                            })
                        });
                        if let Some(id) = id {
                            if !entry["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false) {
                                entry["id"] = Value::String(id);
                            }
                        }
                        if let Some(name) = name {
                            if !entry["function"]["name"].as_str().map(|s| !s.is_empty()).unwrap_or(false) {
                                entry["function"]["name"] = Value::String(name);
                            }
                        }
                        if !arguments_chunk.is_empty() {
                            let current = entry["function"]["arguments"].as_str().unwrap_or("");
                            entry["function"]["arguments"] = Value::String(format!("{}{}", current, arguments_chunk));
                        }
                    }
                    StreamEvent::ToolCallReady { id, name, arguments } => {
                        has_tool_calls = true;
                        let key = format!("tc_ready_{}", id);
                        tool_calls_map.insert(key, serde_json::json!({
                            "id": id, "type": "function",
                            "function": {"name": name, "arguments": arguments}
                        }));
                    }
                    #[allow(unreachable_patterns)]
                    StreamEvent::Done { finish_reason: fr } => {
                        finish_reason = fr;
                    }
                    StreamEvent::Usage { input_tokens, output_tokens, cache_hit, cache_miss } => {
                        self.total_input_tokens += input_tokens;
                        self.total_output_tokens += output_tokens;
                        self.last_cache_hit += cache_hit;
                        self.last_cache_miss += cache_miss;
                    }
                }
            }

            if !full_reasoning.is_empty() {
                let _ = self.app_handle.emit("reasoning:done", serde_json::json!({"full": full_reasoning}));
            }

            let fr = if finish_reason.is_empty() { "stop" } else { &finish_reason };
            let has_parseable_tool_calls = has_tool_calls && !tool_calls_map.is_empty();

            if fr == "tool_calls" && has_parseable_tool_calls {
                let parsed_tcs: Vec<ApiToolCall> = tool_calls_map.into_values()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect();

                if parsed_tcs.is_empty() {
                    final_text = full_content.clone();
                    self.conversation_history.push(ChatMessage {
                        role: "assistant".into(), content: Some(full_content),
                        tool_calls: None, tool_call_id: None,
                    });
                    break;
                }

                // Mode enforcement: reject write tools in Plan mode
                let is_plan = mode.map(|m| m.eq_ignore_ascii_case("plan")).unwrap_or(false);
                if is_plan {
                    let has_blocked = parsed_tcs.iter().any(|tc| {
                        tools::check_mode_tool_allowed(&tc.function.name, mode).is_err()
                    });
                    if has_blocked {
                        for tc in &parsed_tcs {
                            let ok = tools::check_mode_tool_allowed(&tc.function.name, mode).is_ok();
                            let output = if ok {
                                match tools::dispatch_tool_call(&self.tool_registry, &tc.function.name, &tc.function.arguments).await {
                                    Ok(o) => o,
                                    Err(e) => format!("Error: {e}"),
                                }
                            } else {
                                format!("Blocked by Plan mode: {} is read-only", tc.function.name)
                            };
                            let _ = self.app_handle.emit("tool:end", serde_json::json!({
                                "id": tc.id, "name": tc.function.name, "output": output.clone(), "success": ok,
                            }));
                            if tc.function.name == "update_plan" {
                                if let Some(plan) = crate::tools::get_latest_plan() {
                                    let _ = self.app_handle.emit("plan:updated", serde_json::json!({
                                        "plan": plan,
                                    }));
                                }
                            }
                        }
                        self.conversation_history.push(ChatMessage {
                            role: "assistant".into(),
                            content: if full_content.is_empty() { None } else { Some(full_content.clone()) },
                            tool_calls: None, tool_call_id: None,
                        });
                        continue;
                    }
                }

                for tc in &parsed_tcs {
                    let _ = self.app_handle.emit("tool:start", serde_json::json!({
                        "id": tc.id, "name": tc.function.name, "arguments": tc.function.arguments,
                    }));
                }

                let futs: Vec<_> = parsed_tcs.iter().map(|tc| {
                    tools::dispatch_tool_call(&self.tool_registry, &tc.function.name, &tc.function.arguments)
                }).collect();
                let results = futures::future::join_all(futs).await;

                let mut api_tool_calls = Vec::new();
                let mut tool_results = Vec::new();

                for (tc, result) in parsed_tcs.iter().zip(results.iter()) {
                    api_tool_calls.push(tc.clone());
                    match result {
                        Ok(output) => {
                            let _ = self.app_handle.emit("tool:end", serde_json::json!({
                                "id": tc.id, "name": tc.function.name, "output": output, "success": true,
                            }));
                            // Emit plan:updated event when the plan changes
                            if tc.function.name == "update_plan" {
                                if let Some(plan) = crate::tools::get_latest_plan() {
                                    let _ = self.app_handle.emit("plan:updated", serde_json::json!({
                                        "plan": plan,
                                    }));
                                }
                            }
                            tool_results.push((tc.id.clone(), output.clone()));

                            // Desktop notification for long-running tools
                            if matches!(tc.function.name.as_str(), "exec_shell" | "run_tests" | "js_execution") {
                                self.send_notification("Tool completed", &format!("{} finished successfully", tc.function.name));
                            }

                            // LSP diagnostics: auto-detect issues after write_file/edit_file
                            if tc.function.name == "write_file" || tc.function.name == "edit_file" {
                                if let Ok(args) = serde_json::from_str::<Value>(&tc.function.arguments) {
                                    if let Some(path_str) = args.get("path").and_then(|v| v.as_str()) {
                                        let file_path = std::path::PathBuf::from(path_str);
                                        if let Some(ref lsp) = self.lsp_manager {
                                            if let Some(diag_block) = lsp.diagnostics_for(&file_path, 0).await {
                                                let diag_text = render_blocks(&[diag_block]);
                                                if !diag_text.is_empty() {
                                                    tracing::debug!(file = %file_path.display(), "lsp diagnostics found");
                                                    self.conversation_history.push(ChatMessage {
                                                        role: "system".into(),
                                                        content: Some(diag_text),
                                                        tool_calls: None, tool_call_id: None,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let err = format!("Error: {e}");
                            let _ = self.app_handle.emit("tool:end", serde_json::json!({
                                "id": tc.id, "name": tc.function.name, "output": err, "success": false,
                            }));
                            tool_results.push((tc.id.clone(), err));
                            self.send_notification("Tool failed", &format!("{} failed: {}", tc.function.name, e));
                        }
                    }
                }

                self.conversation_history.push(ChatMessage {
                    role: "assistant".into(),
                    content: if full_content.is_empty() { None } else { Some(full_content.clone()) },
                    tool_calls: Some(api_tool_calls), tool_call_id: None,
                });
                for (id, output) in tool_results {
                    self.conversation_history.push(ChatMessage {
                        role: "tool".into(), content: Some(output),
                        tool_calls: None, tool_call_id: Some(id),
                    });
                }

                // Post-turn snapshot
                self.turn_counter += 1;
                if let Some(ref ws) = self.workspace {
                    if let Some(ref sm) = self.snapshot_manager {
                        sm.post_turn_snapshot(ws, self.turn_counter).await;
                    }
                }

            } else {
                final_text = full_content.clone();
                self.conversation_history.push(ChatMessage {
                    role: "assistant".into(), content: Some(full_content),
                    tool_calls: None, tool_call_id: None,
                });
                break;
            }
        }
        Ok(final_text)
    }
}