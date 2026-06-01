//! `&mark` — 对话标记/书签
//!
//! 在对话中打标记，方便后续回溯。
//! - `&mark 标签`       → 在当前位置打标记
//! - `&mark list`       → 列出所有标记
//! - `&mark goto <N>`   → 回溯到标记位置

use std::fmt::Write;
use std::path::PathBuf;

use chrono::Local;

use crate::session_manager::create_saved_session_with_mode;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 标记文件目录名
const MARKS_DIR: &str = ".codewhale/marks";

/// 打标记
pub fn mark(app: &mut App, arg: Option<&str>) -> CommandResult {
    let label = arg.map(str::trim).filter(|s| !s.is_empty());

    match label {
        None => {
            if app.api_messages.is_empty() {
                return CommandResult::error("还没有对话，无法打标记。");
            }
            CommandResult::with_message_and_action(
                "[标记] AI 正在总结当前进展...",
                AppAction::SendMessage(
                    "请用一句话总结当前对话的关键进展和已完成的成果，\
                     不要包含计划或待办事项。\
                     只说已经做了什么。".to_string(),
                ),
            )
        }
        Some("list" | "ls") => list_marks(app),
        Some(n) if n.parse::<usize>().is_ok() => jump_to_mark(app, n),
        Some(label) => create_mark(app, label),
    }
}

/// 创建标记（保存当前会话到 marks 目录）
fn create_mark(app: &mut App, label: &str) -> CommandResult {
    if app.api_messages.is_empty() {
        return CommandResult::error("还没有对话，无法打标记。");
    }

    let mut session = create_saved_session_with_mode(
        &app.api_messages,
        &app.model,
        &app.workspace,
        u64::from(app.session.total_tokens),
        app.system_prompt.as_ref(),
        Some(app.mode.label()),
    );

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    session.metadata.title = format!("[标记] {label} @ {timestamp}");

    let session_path = get_marks_dir(app).join(format!("{}.json", &session.metadata.id));
    if let Some(parent) = session_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return CommandResult::error(format!("无法创建标记目录：{e}"));
        }
    }
    match std::fs::write(&session_path, serde_json::to_string_pretty(&session).unwrap()) {
        Ok(()) => CommandResult::message(format!("[标记] 已打标记「{label}」")),
        Err(e) => CommandResult::error(format!("保存标记失败：{e}")),
    }
}

/// 列出标记
fn list_marks(app: &App) -> CommandResult {
    let marks_dir = get_marks_dir(app);
    if !marks_dir.exists() {
        return CommandResult::message("暂无标记。对话中输入 `&mark <标签>` 打标记。");
    }

    let mut marks: Vec<(usize, String, String, PathBuf)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&marks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let title = val["metadata"]["title"]
                        .as_str()
                        .unwrap_or("(无标签)")
                        .to_string();
                    let ts = val["metadata"]["created_at"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    marks.push((marks.len() + 1, title, ts, path));
                }
            }
        }
    }

    if marks.is_empty() {
        return CommandResult::message("暂无标记。对话中输入 `&mark <标签>` 打标记。");
    }

    marks.sort_by(|a, b| a.2.cmp(&b.2));

    let mut out = String::from("[标记] 标记列表\n\n");
    for (i, (_, title, ts, _)) in marks.iter().enumerate() {
        let n = i + 1;
        let _ = writeln!(out, "  #{n:<2}  {title}  {ts}");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "使用 `&mark <编号>` 回溯到对应标记位置。");

    CommandResult::message(out)
}

/// 跳转到标记位置
fn jump_to_mark(app: &App, n_str: &str) -> CommandResult {
    let n: usize = match n_str.parse() {
        Ok(n) if n >= 1 => n,
        _ => return CommandResult::error("用法：&mark <编号>（编号从 1 开始）"),
    };

    let marks_dir = get_marks_dir(app);
    let mut sessions: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&marks_dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path().metadata().map(|m| m.modified().ok()).ok().flatten());
        for entry in entries {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }
            sessions.push(path);
        }
    }

    if sessions.is_empty() || n > sessions.len() {
        return CommandResult::error(format!(
            "标记 #{n} 不存在，当前共 {} 个标记。",
            sessions.len()
        ));
    }

    let path = sessions[n - 1].clone();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return CommandResult::error(format!("无法读取标记文件：{e}")),
    };
    let saved: crate::session_manager::SavedSession = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => return CommandResult::error(format!("标记文件格式错误：{e}")),
    };
    CommandResult::with_message_and_action(
        format!("[回溯] 正在回溯到标记 #{n}..."),
        AppAction::SyncSession {
            session_id: Some(saved.metadata.id),
            messages: saved.messages,
            system_prompt: saved.system_prompt.map(crate::models::SystemPrompt::Text),
            model: saved.metadata.model,
            workspace: saved.metadata.workspace,
        },
    )
}

/// 获取 marks 目录路径
fn get_marks_dir(app: &App) -> PathBuf {
    app.workspace.join(MARKS_DIR)
}
