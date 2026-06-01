//! `&glance` — 智能项目速览
//!
//! 让模型分析项目并返回一段简洁的概况。
//! 不写文件，纯输出。跟 `&init` 思路一致，但只输出不写。

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 项目速览入口
pub fn glance(app: &App) -> CommandResult {
    CommandResult::with_message_and_action(
        "[速览] AI 正在分析项目概况...",
        AppAction::SendMessage(build_glance_prompt(app)),
    )
}

/// 构造速览 prompt
fn build_glance_prompt(_app: &App) -> String {
    String::from(
        r#"请为当前项目生成一段简明的项目速览。

## 任务

用工具了解项目（读 Cargo.toml / package.json / README / 目录结构等），
然后输出以下信息（保持简短，不要冗余）：

### 1. 项目类型
一行：项目类型 + 一句话说明

### 2. 核心命令
- 构建命令
- 测试命令
- 运行命令

### 3. 技术栈
使用的语言、框架、关键依赖

### 4. 架构快照
2-3 个关键模块及其作用

### 5. 一句话总结
这个项目是做什么的

---

直接输出结果，不要使用工具来生成文件，只需要在回答中呈现。
"#,
    )
}
