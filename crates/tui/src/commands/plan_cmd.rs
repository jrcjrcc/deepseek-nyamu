//! `&plan <prompt>` — 智能规划指令
//!
//! 指导模型按 5 阶段流程制定计划：
//! 探索 → 设计 → 审查 → 写计划 → 等待审批

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 规划入口
pub fn plan(_app: &App, arg: Option<&str>) -> CommandResult {
    let task = match arg.map(str::trim).filter(|s| !s.is_empty()) {
        Some(t) => t,
        None => return CommandResult::error("用法：&plan <要规划的任务>"),
    };

    let message = format!(
        r#"请按以下流程制定规划，**不要动手执行**，等用户确认后再开始。

## 阶段 1：理解需求
用 explore 子代理阅读相关代码，理解需求。

## 阶段 2：设计方案
用 plan 子代理设计方案。

## 阶段 3：审查
确认方案符合需求。有疑问用 `request_user_input` 问用户。

## 阶段 4：记录计划
调用 `update_plan` 工具（你的工具列表里就有）记录计划：
- explanation：总体说明
- plan：步骤列表，每条含 step（描述）和 status（设为 "pending"）

## 阶段 5：等待确认
总结方案后告诉用户"方案已就绪"，等用户说"开始"再执行。

用户的任务：{task}"#,
        task = task
    );

    CommandResult::with_message_and_action(
        format!("[计划] 正在为「{task}」制定计划..."),
        AppAction::SendMessage(message),
    )
}
