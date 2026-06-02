//! `&dream`, `&tips`, `&traps`, `&update` — 记忆与项目指令相关命令。
//!
//! - `&dream` — 启动 Dream 子代理进行四阶段记忆整合
//! - `&tips` — 将 tips.md 内容注入到上下文
//! - `&traps` — 将 traps.md 内容注入到上下文
//! - `&update` — 分析当前会话，总结改动和思路到项目指令文件

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::tui::app::{App, AppAction};

use super::CommandResult;
use codewhale_core::dream::config::DreamConfig;
use codewhale_core::dream::lock::LockStatus;

fn memory_dir(_app: &App) -> PathBuf {
    DreamConfig::default().resolved_memory_dir()
}

/// 读取记忆文件内容用于注入
fn read_memory_file(filename: &str, app: &App) -> Option<String> {
    let path = memory_dir(app).join(filename);
    let content = fs::read_to_string(&path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content)
}

/// 处理 `&dream` 命令
///
/// 在启动子代理之前先获取文件锁，防止多个 `&dream` 同时运行导致
/// TOCTOU 竞态问题（记忆文件损坏）。
pub fn dream(app: &App, arg: Option<&str>) -> CommandResult {
    let _ = arg;

    // 获取记忆目录路径并尝试获取整合锁
    let memory_dir = memory_dir(app);
    match codewhale_core::dream::lock::ConsolidationLock::try_acquire(
        &memory_dir,
        Duration::from_secs(3600), // 锁超时 1 小时
    ) {
        Ok(LockStatus::Acquired(_lock)) => {
            // 锁将在函数返回时自动释放（Drop）
        }
        Ok(LockStatus::Held) => {
            return CommandResult::error("另一个记忆整合正在运行，请等待其完成后再试。");
        }
        Err(e) => {
            return CommandResult::error(format!("无法获取记忆整合锁（内部错误）：{}", e,));
        }
    }

    let prompt = codewhale_core::dream::consolidator::build_prompt(&DreamConfig::default());

    let message = format!(
        "用户手动触发了记忆整合。请立即执行以下任务：\n\n\
         {prompt}\n\n\
         使用 `agent_open` 启动子代理执行整合，工具白名单：\
         read_file, write_file, edit_file, grep_files, list_dir。\
         完成后输出摘要。"
    );

    CommandResult::with_message_and_action(
        "🧠 启动 Dream 记忆整合...",
        AppAction::SendMessage(message),
    )
}

/// 处理 `&tips` 命令
pub fn tips(app: &App, _arg: Option<&str>) -> CommandResult {
    match read_memory_file("tips.md", app) {
        Some(content) => {
            let message = format!(
                "以下是 tips.md 记忆文件的内容，请参考其中的技巧和配置信息：\n\n\
                 ---\n{content}\n---\n\n# 用户请求\n\n请结合以上技巧信息回答。"
            );
            CommandResult::with_message_and_action(
                "📋 注入 tips.md...",
                AppAction::SendMessage(message),
            )
        }
        None => CommandResult::error("tips.md 不存在或为空"),
    }
}

/// 处理 `&update` 命令 — 分析会话总结到项目指令文件
pub fn update(app: &App, _arg: Option<&str>) -> CommandResult {
    // 探测项目指令文件（WHALE.md > .codewhale/instructions.md > AGENTS.md）
    let candidates = [
        app.workspace.join("WHALE.md"),
        app.workspace.join(".codewhale").join("instructions.md"),
        app.workspace.join("AGENTS.md"),
    ];

    let (target_path, existing_content) = candidates
        .iter()
        .find_map(|p| {
            let content = fs::read_to_string(p).ok()?;
            Some((p.clone(), content))
        })
        .unwrap_or_else(|| {
            // 默认用 WHALE.md，不存在则新建
            (candidates[0].clone(), String::new())
        });

    let path_str = target_path.display().to_string();
    let session_summary_prompt = format!(
        "请分析本次会话的完整内容，提取以下信息并更新到 `{path_str}`：\n\n\
         1. **本次改动**：所有对代码、配置、文档的修改（新增/删除/修复）\n\
         2. **设计思路**：做出的关键决策和技术选型理由\n\
         3. **项目知识点**：新发现的项目相关知识、陷阱、技巧\n\
         4. **待办事项**：需要未来处理的问题或改进\n\n\
         当前 `{path_str}` 内容：\n\
         ---\n{}\n---\n\n\
         请用 `read_file` 先读取当前文件确认内容，然后用 `edit_file` 或 `write_file` 更新文件。\
         更新的格式要保持原有结构，在合适的位置追加新内容。\
         如果文件不存在则用 `write_file` 创建。\
         完成后输出摘要说明更新了什么。",
        if existing_content.is_empty() {
            "(文件不存在，将创建)".to_string()
        } else {
            existing_content
        }
    );

    CommandResult::with_message_and_action(
        "📝 分析会话并更新项目指令文件...",
        AppAction::SendMessage(session_summary_prompt),
    )
}

/// 处理 `&traps` 命令
pub fn traps(app: &App, _arg: Option<&str>) -> CommandResult {
    match read_memory_file("traps.md", app) {
        Some(content) => {
            let message = format!(
                "以下是 traps.md 记忆文件的内容，请参考已知的陷阱和踩坑记录：\n\n\
                 ---\n{content}\n---\n\n# 用户请求\n\n请结合以上踩坑记录回答，避免重复踩坑。"
            );
            CommandResult::with_message_and_action(
                "⚠️ 注入 traps.md...",
                AppAction::SendMessage(message),
            )
        }
        None => CommandResult::error("traps.md 不存在或为空"),
    }
}
