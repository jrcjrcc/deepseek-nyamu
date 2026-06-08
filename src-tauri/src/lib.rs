//! DeepWhale 后端核心 crate
//!
//! 提供两种运行模式：
//! - `run_gui()`：启动 Tauri 桌面应用，注册 IPC 命令和事件
//! - `run_cli_mode()`：启动 CLI 交互式或批处理模式
//!
//! 模块组织：
//! - `commands`：所有 Tauri IPC 命令处理函数
//! - `engine`：核心 Agent 引擎（LLM 通信 + 工具调度循环）
//! - `cli`：CLI 子命令解析与执行
//! - `prompts`：系统提示词组装（constitution + mode + personality + skills）
//! - `tools`：内置工具注册与分发
//! - `sandbox`：沙盒执行策略
//! - `auto_router`：自动模型路由
//! - `memory`：持久化用户记忆
//! - `project_context`：项目上下文注入
//! - `lsp`：LSP 诊断集成
//! - `snapshot`：文件系统快照（用于撤销）
//! - `rlm`：递归语言模型（长文本分解）
//! - `swebench`：SWE-bench 评估导出
//! - `skills`：Skill 启用/禁用管理
//! - `mobile_page`：移动端页面（预留）
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

mod auto_router;
mod cli;
mod commands;
mod engine;
mod lsp;
mod memory;
mod mobile_page;
mod project_context;
mod prompts;
mod rlm;
mod sandbox;
mod snapshot;
mod skills;
pub mod swebench;
pub mod tools;

use std::collections::HashMap;

/// 子代理运行时状态
///
/// 通过 `agent_open`/`agent_eval`/`agent_close` 命令管理。
/// 每个子代理在独立的 tokio 任务中运行，
/// 支持轮询查询状态（`agent_eval`）和清理（`agent_close`）。
pub struct SubAgentState {
    pub prompt: String,                            // 子代理的提示词
    pub status: String, // "running" | "done" | "error"  // 当前状态
    pub result: Option<String>,                    // 执行结果（完成时）
    pub error: Option<String>,                     // 错误信息（失败时）
    pub handle: Option<tokio::task::JoinHandle<()>>, // tokio 任务句柄
}

/// 全局应用状态（由 Tauri 管理）
///
/// 通过 `tauri::Builder::setup` 中 `app.manage()` 注册，
/// 在 Tauri 命令中通过 `State<AppState>` 注入。
pub struct AppState {
    pub engine: Arc<Mutex<engine::DeepWhaleEngine>>,            // 核心引擎
    pub subagents: Arc<tokio::sync::RwLock<HashMap<String, SubAgentState>>>, // 子代理池
}

/// GUI 模式入口 —— 启动 Tauri 桌面应用
///
/// 初始化流程：
/// 1. 设置 tracing 日志（支持 RUST_LOG 环境变量）
/// 2. 注册 tauri-plugin-shell 插件
/// 3. 创建 DeepWhaleEngine 实例并注册为全局状态
/// 4. 注册所有 IPC 命令处理函数
pub fn run_gui() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let engine = engine::DeepWhaleEngine::new(app_handle.clone());
            app.manage(AppState {
                engine: Arc::new(Mutex::new(engine)),
                subagents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::submit_message,
            commands::get_sessions,
            commands::get_config,
            commands::update_config,
            commands::get_conversation,
            commands::new_session,
            commands::rename_session,
            commands::delete_session,
            commands::list_personalities,
            commands::set_personality,
            commands::get_current_personality,
            commands::list_skills,
            commands::exec_shell_direct,
            commands::get_session_usage,
            commands::agent_open,
            commands::agent_eval,
            commands::agent_close,
            commands::get_mobile_page,
            commands::get_memory,
            commands::get_subagents,
            commands::list_models,
            commands::set_model,
            commands::set_effort,
            commands::list_directory_tree,
            commands::read_file_content,
            commands::get_session_changes,
            commands::needs_onboarding,
            commands::connect_key,
            commands::get_system_prompt,
            commands::purge_context,
            commands::get_balance,
            commands::get_cache_stats,
            commands::export_session,
            commands::save_session_file,
            commands::load_session_file,
            commands::toggle_lsp,
            commands::get_workspace_diff,
            commands::get_workspace_info,
            commands::get_session_plans,
            commands::get_latest_plan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DeepWhale");
}

/// CLI 模式入口 —— 解析参数并执行终端子命令
///
/// 支持子命令：sessions, login, logout, auth, config, models,
/// doctor, update, metrics, completion, sandbox, serve, version,
/// setup, exec, resume, fork, mcp-server, mcp-validate, run-pr,
/// swebench, rlm, skill, memory
pub fn run_cli_mode() {
    use clap::Parser;
    let cli = cli::Cli::parse();

    if let Err(err) = cli::run_cli(&cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
  
