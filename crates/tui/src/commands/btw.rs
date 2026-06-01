//! `&&` 旁路提问——不打断主对话的侧边问题。
//!
//! 用法：
//! - `&&你好` → 开子代理问"你好"
//! - `&&`     → 问"请详细阐述模型现在正在执行什么任务"

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 旁路提问的默认问题
const DEFAULT_QUESTION: &str = "请详细阐述模型现在正在执行什么任务";

/// 处理 `&&` 旁路提问
///
/// 构造一条指令让模型通过 `agent_open` 开子代理回答，
/// 不中断主对话。
pub fn ask(app: &App, question: &str) -> CommandResult {
    let question = if question.is_empty() {
        DEFAULT_QUESTION.to_string()
    } else {
        question.to_string()
    };

    let message = format!(
        "用户发了一个旁路提问，不打断当前任务。\
         请用 `agent_open` 开一个子代理来回答：\n\n\
         {question}\n\n\
         子代理完成后用 `agent_eval` 获取回答，\
         然后用 `handle_read` 读完整结果。\
         验证回答后再报告给我。"
    );

    CommandResult::with_message_and_action(
        format!("[旁路] {question}"),
        AppAction::SendMessage(message),
    )
}
