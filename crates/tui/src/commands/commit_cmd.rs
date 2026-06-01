//! `&commit` — 智能 git 提交
//!
//! 分析当前变更 → 生成 commit message → 提交。
//! 通过模型生成消息，用 git add + git commit 完成。

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 提交入口
pub fn commit(_app: &App, arg: Option<&str>) -> CommandResult {
    let extra = arg.map(str::trim).filter(|s| !s.is_empty());

    let mut message = String::from(
        "分析当前 Git 变更并创建一个规范的 commit。\n\n\
         1. 执行 `git status` 和 `git diff HEAD` 了解变更\n\
         2. 查看 `git log --oneline -10` 了解提交风格\n\
         3. 生成清晰的 commit message（type(scope): 描述）\n\
         4. 用 `git add` 暂存相关文件\n\
         5. 用 `git commit -m \"...\"` 提交\n\n\
         重要规则：\n\
         - 不要修改 git config\n\
         - 不要 --amend，除非用户明确要求\n\
         - 不要提交 .env 等敏感文件\n\
         - 不要创建空 commit\n\
         - 不要用 -i 交互模式\n"
    );

    if let Some(msg) = extra {
        message.push_str(&format!("\n用户补充说明：{msg}\n"));
    }

    CommandResult::with_message_and_action(
        "[提交] 正在分析变更并创建 commit...",
        AppAction::SendMessage(message),
    )
}
