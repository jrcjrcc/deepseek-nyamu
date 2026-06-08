//! Tauri IPC 命令处理函数
//!
//! 所有通过 `invoke_handler` 注册的 Tauri 命令在此实现。
//! 命令分类：
//! - 会话管理：submit_message, get_sessions, get_conversation, new_session
//! - 配置管理：get_config, update_config
//! - 人格设置：list_personalities, set_personality, get_current_personality
//! - 技能管理：list_skills
//! - Shell 执行：exec_shell_direct
//! - 用量统计：get_session_usage
//! - 子代理管理：agent_open, agent_eval, agent_close
//!
//! 每个命令通过 `#[tauri::command]` 宏标记，Tauri 自动处理序列化。
use std::collections::HashMap;
use tauri::State;
use crate::AppState;
use crate::prompts;

/// 提交用户消息并触发 AI 回复流程
///
/// 会触发一系列 Tauri 事件（token, reasoning, tool:start, tool:end），
/// 前端通过这些事件流实时渲染 AI 的回复过程。
/// mode 参数控制工具可用性（agent / plan / yolo）。
#[tauri::command]
pub async fn submit_message(
    state: State<'_, AppState>,
    session_id: String,
    content: String,
    mode: Option<String>,
) -> Result<String, String> {
    let mut engine = state.engine.lock().await;
    engine.submit_message_with_mode(&session_id, &content, mode.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 获取所有会话列表
#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<crate::engine::SessionInfo>, String> {
    let engine = state.engine.lock().await;
    Ok(engine.sessions.clone())
}

/// 读取当前配置（JSON 格式的键值对）
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let engine = state.engine.lock().await;
    let json = engine.config_store.as_ref()
        .and_then(|cs| {
            let values = cs.config.list_values();
            serde_json::to_string_pretty(&values).ok()
        })
        .unwrap_or_else(|| "{}".to_string());
    Ok(json)
}

/// 更新配置（接收 JSON 格式的键值对映射）
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config_json: String,
) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    if let Some(store) = &mut engine.config_store {
        let updates: HashMap<String, String> =
            serde_json::from_str(&config_json).map_err(|e| e.to_string())?;
        for (key, value) in updates {
            store.config.set_value(&key, &value).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 获取指定会话的对话历史
/// 从 engine 的内存中会话历史转换
#[tauri::command]
pub async fn get_conversation(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<crate::engine::Message>, String> {
    let engine = state.engine.lock().await;
    // Check if this is the active session
    if engine.active_session_id.as_deref() == Some(&session_id) {
        let msgs: Vec<crate::engine::Message> = engine.conversation_history.iter().map(|cm| {
            let tool_calls = cm.tool_calls.as_ref().map(|tcs| {
                tcs.iter().map(|tc| crate::engine::ToolCallInfo {
                    name: tc.function.name.clone(),
                    input: tc.function.arguments.clone(),
                    output: None,
                    status: "completed".to_string(),
                }).collect()
            });
            crate::engine::Message {
                role: cm.role.clone(),
                content: cm.content.clone().unwrap_or_default(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tool_calls,
            }
        }).collect();
        Ok(msgs)
    } else {
        Ok(Vec::new())
    }
}

// Personality commands
#[tauri::command]
pub fn list_personalities() -> Vec<String> {
    prompts::list_personalities()
}
#[tauri::command]
pub fn set_personality(name: String) -> Result<(), String> {
    prompts::set_personality(&name)
}
#[tauri::command]
pub fn get_current_personality() -> String {
    prompts::get_personality_name()
}

// Skill commands
#[tauri::command]
pub fn list_skills() -> Vec<serde_json::Value> {
    crate::skills::list_skills()
        .into_iter()
        .map(|s| serde_json::json!({"name": s.name, "description": s.description, "path": s.path}))
        .collect()
}

// Shell command (for ! prefix)
#[tauri::command]
#[allow(dead_code)]
pub async fn exec_shell_direct(command: String, cwd: Option<String>) -> Result<String, String> {
    use std::process::Command;
    #[cfg(windows)]
    let mut cmd = { let mut c = Command::new("powershell"); c.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", "& {", &command, "}"]); c };
    #[cfg(not(windows))]
    let mut cmd = { let mut c = Command::new("sh"); c.args(["-c", &command]); c };
    if let Some(dir) = cwd { if !dir.is_empty() { cmd.current_dir(&dir); } }
    let output = cmd.output().map_err(|e| format!("{e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() { stdout.to_string() } else { format!("{stdout}\n{stderr}") };
    if !output.status.success() { Err(format!("exit code {}\n{combined}", output.status.code().unwrap_or(-1))) }
    else { Ok(combined) }
}

// Session usage
#[derive(serde::Serialize)]
pub struct SessionUsage {
    pub input_tokens: u64, pub output_tokens: u64, pub total_cost: f64, pub cache_hit_rate: f64,
}
#[tauri::command]
pub async fn get_session_usage(state: State<'_, AppState>) -> Result<SessionUsage, String> {
    let engine = state.engine.lock().await;
    Ok(SessionUsage { input_tokens: engine.total_input_tokens, output_tokens: engine.total_output_tokens, total_cost: engine.total_cost, cache_hit_rate: engine.cache_hit_rate() })
}

// Session commands
#[tauri::command]
pub async fn new_session(state: State<'_, AppState>, title: String) -> Result<crate::engine::SessionInfo, String> {
    let mut engine = state.engine.lock().await;
    let session = crate::engine::SessionInfo {
        id: uuid::Uuid::new_v4().to_string(), title, created_at: chrono::Utc::now().to_rfc3339(), message_count: 0,
    };
    engine.sessions.push(session.clone());
    engine.active_session_id = Some(session.id.clone());
    Ok(session)
}

#[tauri::command]
pub async fn rename_session(state: State<'_, AppState>, session_id: String, title: String) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    if let Some(s) = engine.sessions.iter_mut().find(|s| s.id == session_id) {
        s.title = title;
        Ok(())
    } else {
        Err(format!("Session '{session_id}' not found"))
    }
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    let before = engine.sessions.len();
    engine.sessions.retain(|s| s.id != session_id);
    if engine.sessions.len() < before {
        if engine.active_session_id.as_deref() == Some(&session_id) {
            engine.active_session_id = engine.sessions.first().map(|s| s.id.clone());
        }
        Ok(())
    } else {
        Err(format!("Session '{session_id}' not found"))
    }
}

#[tauri::command]
pub fn get_mobile_page() -> String {
    crate::mobile_page::get_mobile_page_html()
}

#[tauri::command]
pub fn get_memory() -> Result<String, String> {
    let path = crate::memory::resolve_global_memory_path(None);
    crate::memory::read_memory(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_subagents(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let sa = state.subagents.read().await;
    Ok(sa.iter().map(|(id, s)| {
        serde_json::json!({"id": id, "status": s.status, "result": s.result, "error": s.error, "prompt": s.prompt})
    }).collect())
}

// Sub-agent commands
#[tauri::command]
pub async fn agent_open(state: State<'_, AppState>, prompt: String, model: Option<String>, api_key: Option<String>) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    state.subagents.write().await.insert(id.clone(), crate::SubAgentState {
        prompt: prompt.clone(), status: "running".into(), result: None, error: None, handle: None,
    });

    let engine = state.engine.lock().await;
    let key = api_key.unwrap_or_else(|| {
        if let Some(k) = engine.config_store.as_ref().and_then(|s| s.config.api_key.as_deref()) {
            if !k.is_empty() { return k.to_string(); }
        }
        std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()
    });
    let base_url = engine.config_store.as_ref()
        .and_then(|s| s.config.providers.for_provider(nyamu_config::ProviderKind::Deepseek).base_url.as_deref())
        .map(|s| s.to_string()).unwrap_or_else(|| "https://api.deepseek.com/beta".to_string());
    let mdl = model.unwrap_or_else(|| "deepseek-v4-flash".to_string());
    drop(engine);

    let subagents = state.subagents.clone();
    let cid = id.clone();
    tokio::spawn(async move {
        let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(300)).build() {
            Ok(c) => c, Err(e) => {
                let mut sa = subagents.write().await;
                if let Some(s) = sa.get_mut(&cid) { s.status = "error".into(); s.error = Some(e.to_string()); }
                return;
            }
        };
        let body = serde_json::json!({"model": mdl, "messages": [{"role": "user", "content": prompt}], "max_tokens": 8192, "stream": false});
        let resp = client.post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&body).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(data) = r.json::<serde_json::Value>().await {
                    let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                    let mut sa = subagents.write().await;
                    if let Some(s) = sa.get_mut(&cid) { s.status = "done".into(); s.result = Some(content); }
                }
            }
            Ok(r) => {
                let mut sa = subagents.write().await;
                if let Some(s) = sa.get_mut(&cid) { s.status = "error".into(); s.error = Some(format!("HTTP {}", r.status())); }
            }
            Err(e) => {
                let mut sa = subagents.write().await;
                if let Some(s) = sa.get_mut(&cid) { s.status = "error".into(); s.error = Some(e.to_string()); }
            }
        }
    });
    Ok(id)
}

#[tauri::command]
pub async fn agent_eval(state: State<'_, AppState>, session_id: String) -> Result<serde_json::Value, String> {
    let sa = state.subagents.read().await;
    match sa.get(&session_id) {
        Some(s) => Ok(serde_json::json!({"id": session_id, "status": s.status, "result": s.result, "error": s.error, "prompt": s.prompt})),
        None => Err(format!("Sub-agent '{session_id}' not found")),
    }
}

#[tauri::command]
pub async fn agent_close(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.subagents.write().await.remove(&session_id);
    Ok(())
}

/* ─── 模型/Effort 切换 ────────────────────────────────── */

#[tauri::command]
pub fn list_models(state: State<'_, AppState>) -> Vec<serde_json::Value> {
    let mut models: Vec<serde_json::Value> = Vec::new();
    if let Ok(engine) = state.engine.try_lock() {
        if let Some(ref cs) = engine.config_store {
            let current = cs.config.model.clone();
            for m in &["deepseek-v4-flash", "deepseek-v3", "deepseek-r1", "deepseek-chat", "deepseek-reasoner"] {
                let is_active = current.as_deref() == Some(m);
                models.push(serde_json::json!({"name": m, "active": is_active}));
            }
        }
    }
    if models.is_empty() {
        models.push(serde_json::json!({"name": "deepseek-v4-flash", "active": true}));
    }
    models
}

#[tauri::command]
pub fn set_model(state: State<'_, AppState>, model_name: String) -> Result<(), String> {
    if let Ok(mut engine) = state.engine.try_lock() {
        if let Some(ref mut cs) = engine.config_store {
            cs.config.set_value("model", &model_name).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_effort(effort: String) -> Result<(), String> {
    let valid = ["off", "low", "medium", "high", "max"];
    if !valid.contains(&effort.as_str()) {
        return Err(format!("Invalid effort: {}. Valid: {}", effort, valid.join(", ")));
    }
    // SAFETY: setting env var is safe in single-threaded command context
    unsafe { std::env::set_var("DEEPSEEK_REASONING_EFFORT", &effort); }
    Ok(())
}

/* ─── 文件浏览 ────────────────────────────────────────── */

#[tauri::command]
pub fn list_directory_tree(path: String) -> Result<serde_json::Value, String> {
    let root = std::path::PathBuf::from(&path);
    if !root.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    scan_dir(&root).map(|entries| serde_json::json!({"path": path, "entries": entries}))
}

/// Recursively scan a directory, returning a JSON tree.
fn scan_dir(dir: &std::path::Path) -> Result<Vec<serde_json::Value>, String> {
    let mut entries = Vec::new();
    let mut rd = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {}", dir.display(), e))?;
    for entry in rd.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden files/dirs
            if name.starts_with('.') { continue; }
            let is_dir = path.is_dir();
            let size = if is_dir { 0 } else { std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) };
            entries.push(serde_json::json!({
                "name": name,
                "path": path.to_string_lossy(),
                "is_dir": is_dir,
                "size": size,
                "children": if is_dir { scan_dir(&path).ok() } else { None },
            }));
        }
        entries.sort_by(|a, b| {
            let ad = a["is_dir"].as_bool().unwrap_or(false);
            let bd = b["is_dir"].as_bool().unwrap_or(false);
            if ad != bd { return bd.cmp(&ad); }
            a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or(""))
        });
        Ok(entries)
    }

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() { return Err(format!("File not found: {}", path)); }
    if !p.is_file() { return Err(format!("Not a file: {}", path)); }
    std::fs::read_to_string(&p).map_err(|e| format!("read {}: {}", path, e))
}

#[tauri::command]
pub async fn get_session_changes(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let ws = {
        let engine = state.engine.lock().await;
        engine.workspace.clone()
    };
    if let Some(ref ws_path) = ws {
        let engine = state.engine.lock().await;
        if let Some(ref sm) = engine.snapshot_manager {
            let snapshots = sm.list_snapshots(ws_path).await;
            return Ok(snapshots.into_iter().map(|s| {
                serde_json::json!({"id": s.id.sha, "label": s.label, "timestamp": s.timestamp})
            }).collect());
        }
    }
    Ok(Vec::new())
}

/* ─── 设置管理 ────────────────────────────────────────── */

#[tauri::command]
pub fn needs_onboarding(state: State<'_, AppState>) -> bool {
    if let Ok(engine) = state.engine.try_lock() {
        if let Some(ref cs) = engine.config_store {
            if cs.config.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false) {
                return false;
            }
        }
    }
    if std::env::var("DEEPSEEK_API_KEY").ok().map(|k| !k.is_empty()).unwrap_or(false) {
        return false;
    }
    true
}

#[tauri::command]
pub fn connect_key(state: State<'_, AppState>, api_key: String) -> Result<(), String> {
    if let Ok(mut engine) = state.engine.try_lock() {
        if let Some(ref mut cs) = engine.config_store {
            cs.config.set_value("api_key", &api_key).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/* ─── 系统提示词 ───────────────────────────────────── */

#[tauri::command]
pub fn get_system_prompt(state: State<'_, AppState>) -> String {
    if let Ok(_engine) = state.engine.try_lock() {
        let personality = crate::prompts::get_personality_name();
        let personality_opt = if personality.is_empty() || personality == "default" {
            None
        } else {
            Some(personality.as_str())
        };
        return crate::prompts::build_system_prompt(personality_opt);
    }
    String::new()
}

/* ─── 上下文管理 ───────────────────────────────────── */

#[tauri::command]
pub async fn purge_context(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.engine.lock().await;
    engine.conversation_history.clear();
    engine.turn_counter = 0;
    // Reset token tracking for the new context window
    engine.total_input_tokens = 0;
    engine.total_output_tokens = 0;
    engine.total_cost = 0.0;
    Ok(())
}

/* ─── 余额查询 ─────────────────────────────────────── */

#[tauri::command]
pub fn get_balance(state: State<'_, AppState>) -> String {
    // Return a placeholder — actual balance requires external API call
    if let Ok(engine) = state.engine.try_lock() {
        let has_key = engine.config_store.as_ref()
            .and_then(|cs| cs.config.api_key.as_deref())
            .map(|k| !k.is_empty())
            .unwrap_or(false);
        if has_key {
            return "API key configured. Use provider dashboard for balance details.".to_string();
        }
    }
    "No API key configured.".to_string()
}

/* ─── 缓存统计 ─────────────────────────────────────── */

#[tauri::command]
pub fn get_cache_stats(state: State<'_, AppState>) -> serde_json::Value {
    if let Ok(engine) = state.engine.try_lock() {
        let hit = engine.last_cache_hit;
        let miss = engine.last_cache_miss;
        let rate = if hit + miss > 0 {
            hit as f64 / (hit + miss) as f64
        } else {
            0.0
        };
        return serde_json::json!({
            "cache_hit": hit,
            "cache_miss": miss,
            "hit_rate": rate,
        });
    }
    serde_json::json!({"cache_hit": 0, "cache_miss": 0, "hit_rate": 0.0})
}

/* ─── 会话导出 ─────────────────────────────────────── */

#[tauri::command]
pub async fn export_session(state: State<'_, AppState>, session_id: String, path: Option<String>) -> Result<String, String> {
    let engine = state.engine.lock().await;
    let mut markdown = String::new();

    // Add session header
    if let Some(session) = engine.sessions.iter().find(|s| s.id == session_id) {
        markdown.push_str(&format!("# Session: {}\n\n", session.title));
        markdown.push_str(&format!("- **ID**: {}\n", session.id));
        markdown.push_str(&format!("- **Created**: {}\n", session.created_at));
        markdown.push_str(&format!("- **Messages**: {}\n\n", session.message_count));
    } else {
        markdown.push_str("# Exported Session\n\n");
    }

    // Format conversation history
    for msg in &engine.conversation_history {
        let role = &msg.role;
        let content = msg.content.clone().unwrap_or_default();
        match role.as_str() {
            "user" => markdown.push_str(&format!("## User\n\n{}\n\n", content)),
            "assistant" => markdown.push_str(&format!("## Assistant\n\n{}\n\n", content)),
            "system" => markdown.push_str(&format!("## System\n\n{}\n\n", content)),
            _ => markdown.push_str(&format!("## {}\n\n{}\n\n", role, content)),
        }
    }

    if let Some(file_path) = path {
        if !file_path.trim().is_empty() {
            let p = std::path::PathBuf::from(&file_path);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            std::fs::write(&p, &markdown).map_err(|e| format!("Failed to write file: {}", e))?;
        }
    }

    Ok(markdown)
}

/* ─── 会话文件保存/加载 ─────────────────────────────── */

#[tauri::command]
pub async fn save_session_file(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let engine = state.engine.lock().await;

    let data = serde_json::json!({
        "sessions": engine.sessions,
        "active_session_id": engine.active_session_id,
        "conversation_history": engine.conversation_history,
        "total_input_tokens": engine.total_input_tokens,
        "total_output_tokens": engine.total_output_tokens,
        "total_cost": engine.total_cost,
    });

    let p = std::path::PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize failed: {}", e))?;
    std::fs::write(&p, &json).map_err(|e| format!("Write failed: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn load_session_file(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("File not found: {}", path));
    }
    let json = std::fs::read_to_string(&p).map_err(|e| format!("Read failed: {}", e))?;

    // Validate JSON structure before acquiring the lock
    let data: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Parse failed: {}", e))?;

    let mut engine = state.engine.lock().await;

    if let Some(sessions) = data["sessions"].as_array() {
        engine.sessions = serde_json::from_value(serde_json::json!(sessions))
            .map_err(|e| format!("Session deserialize failed: {}", e))?;
    }
    engine.active_session_id = data["active_session_id"].as_str().map(String::from);
    if let Some(history) = data["conversation_history"].as_array() {
        engine.conversation_history = serde_json::from_value(serde_json::json!(history))
            .map_err(|e| format!("History deserialize failed: {}", e))?;
    }
    engine.total_input_tokens = data["total_input_tokens"].as_u64().unwrap_or(0);
    engine.total_output_tokens = data["total_output_tokens"].as_u64().unwrap_or(0);
    engine.total_cost = data["total_cost"].as_f64().unwrap_or(0.0);

    Ok(())
}

/* ─── LSP 切换 ──────────────────────────────────────── */

#[tauri::command]
pub fn toggle_lsp(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    if let Ok(engine) = state.engine.try_lock() {
        if engine.lsp_manager.is_none() {
            return Err("LSP manager not initialized".to_string());
        }
        let ws = engine.workspace.clone().unwrap_or_default();
        let cfg = crate::lsp::LspConfig {
            enabled,
            ..crate::lsp::LspConfig::default()
        };
        drop(engine);
        if let Ok(mut engine) = state.engine.try_lock() {
            engine.lsp_manager = Some(crate::lsp::LspManager::new(cfg, ws));
        }
    }
    Ok(())
}

/* ─── 工作区 Diff ───────────────────────────────────── */

#[tauri::command]
pub async fn get_workspace_diff(state: State<'_, AppState>) -> Result<String, String> {
    let ws = {
        let engine = state.engine.lock().await;
        engine.workspace.clone()
    };

    if let Some(ref ws_path) = ws {
        // Try getting git diff if in a git repository
        let git_dir = ws_path.join(".git");
        if git_dir.exists() {
            use std::process::Command;
            let output = Command::new("git")
                .args(["-C", &ws_path.to_string_lossy(), "diff", "--stat"])
                .output()
                .map_err(|e| format!("git diff failed: {}", e))?;
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).to_string();
                if !out.trim().is_empty() {
                    let detail = Command::new("git")
                        .args(["-C", &ws_path.to_string_lossy(), "diff"])
                        .output()
                        .map_err(|e| format!("git diff detail failed: {}", e))?;
                    let detail_out = String::from_utf8_lossy(&detail.stdout).to_string();
                    if detail.status.success() && !detail_out.trim().is_empty() {
                        return Ok(format!("{}\n\n{}", out, detail_out));
                    }
                    return Ok(out);
                }
                return Ok("(no changes)".to_string());
            }
        }
        Ok("(not a git repository or no changes)".to_string())
    } else {
        Ok("(no workspace set)".to_string())
    }
}

/* ─── 工作区信息 ────────────────────────────────────── */

#[tauri::command]
pub fn get_workspace_info(state: State<'_, AppState>) -> serde_json::Value {
    if let Ok(engine) = state.engine.try_lock() {
        let workspace_path = engine.workspace.as_ref().map(|p| p.to_string_lossy().to_string());
        let lsp_enabled = engine.lsp_manager.as_ref().map(|l| l.config().enabled).unwrap_or(false);
        let snapshot_count = engine.snapshot_manager.as_ref()
            .map(|sm| sm.snapshot_count())
            .unwrap_or(0);
        let session_count = engine.sessions.len() as u64;

        return serde_json::json!({
            "path": workspace_path,
            "lsp_enabled": lsp_enabled,
            "snapshot_count": snapshot_count,
            "session_count": session_count,
            "active_session_id": engine.active_session_id,
            "conversation_length": engine.conversation_history.len(),
        });
    }
    serde_json::json!({"path": null, "lsp_enabled": false, "snapshot_count": 0, "session_count": 0})
}

/* ─── 计划管理 ─────────────────────────────────────── */

#[tauri::command]
pub fn get_session_plans() -> Vec<serde_json::Value> {
    crate::tools::get_all_plans()
}

#[tauri::command]
pub fn get_latest_plan() -> Option<serde_json::Value> {
    crate::tools::get_latest_plan()
}
