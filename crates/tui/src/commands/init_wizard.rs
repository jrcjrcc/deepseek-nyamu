//! `&init` — 智能项目初始化向导
//!
//! 让模型用工具分析项目，生成真正的 WHALE.md / AGENTS.md，
//! 而不是简单的模板骨架。参照 Claude Code `/init` 的设计思路。

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// 启动智能初始化
pub fn run_interactive_wizard(app: &App) -> CommandResult {
    CommandResult::with_message_and_action(
        "[初始化] AI 正在分析项目结构并生成指令文件...",
        AppAction::SendMessage(build_init_prompt(app)),
    )
}

/// 构造初始化指令 prompt
fn build_init_prompt(app: &App) -> String {
    // 检测已有的指令文件
    let workspace = &app.workspace;
    let _has_whale = workspace.join("WHALE.md").exists();
    let has_agents = workspace.join("AGENTS.md").exists();
    let has_claude = workspace.join("CLAUDE.md").exists();

    // 决定输出格式：已有优先，否则 WHALE.md
    let output_format = if has_claude {
        "CLAUDE.md"
    } else if has_agents {
        "AGENTS.md"
    } else {
        "WHALE.md"
    };

    format!(
        r#"你需要为当前项目生成一份指令文件，帮助后续的 AI 理解项目。

## 任务

请执行以下步骤并按结果生成内容：

### 步骤 1：了解项目

1. 读取项目的清单文件（Cargo.toml / package.json / pyproject.toml / go.mod 等）
2. 读取 README.md（如果存在）
3. 读取现有的指令文件（WHALE.md / AGENTS.md / CLAUDE.md，如果存在）
4. 列出项目的主要目录结构
5. 检查 CI 配置（.github/workflows/、Makefile、Dockerfile 等）
6. 了解项目的构建、测试、代码检查命令

### 步骤 2：分析架构

- 确定项目的关键模块和入口点
- 识别使用的框架和关键依赖

### 步骤 3：生成 {output_format}

使用 `write_file` 工具将生成的指令写入 `{output_format}`。

## 输出文件格式

```markdown
# 项目指令

## 项目类型

<项目类型>（<项目说明>）

**框架：** <框架列表>

### 命令
- 构建：`<构建命令>`
- 测试：`<测试命令>`
- 代码检查：`<代码检查命令>`
- 运行：`<运行命令>`

---

## 架构

### 关键模块

- **<模块名>**：<说明>

### 入口点

- `<入口路径>`

---

## 代码规范

- <规范说明>

---

## 额外信息

- <补充说明>

---

*由 &init 于 {date} 生成*
```

## 已有内容处理

如果 `{output_format}` 已经存在，先读取现有内容，然后：
1. 保留用户手写的部分（非自动生成的段落）
2. 补充缺失的信息（命令、架构、规范）
3. 更新过时的内容
4. 不要删除用户添加的自定义说明

## 注意事项

- 如果检测到 Rust 项目，检查 Cargo.toml 中的依赖推断框架和规范
- 如果检测到 Node.js 项目，检查 package.json 中的 scripts 和 devDependencies
- 对于 Python 项目，检查 pyproject.toml 中的构建系统和依赖
- 多模块项目（工作空间/monorepo）要列出各个模块的作用
- 不要写空泛的模板内容，每一条都要基于实际项目分析得出
- 不要输出无关的额外文件，只生成指令文件本身
- 如果已有 WHALE.md/AGENTS.md/CLAUDE.md，优先更新而不是覆盖
"#,
        output_format = output_format,
        date = chrono::Local::now().format("%Y-%m-%d %H:%M"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_contains_tasks() {
        let prompt = build_init_prompt_helper();
        assert!(prompt.contains("步骤 1"));
        assert!(prompt.contains("步骤 2"));
        assert!(prompt.contains("步骤 3"));
        assert!(prompt.contains("write_file"));
    }

    #[test]
    fn test_prompt_mentions_readme() {
        let prompt = build_init_prompt_helper();
        assert!(prompt.contains("README"));
    }

    fn build_init_prompt_helper() -> String {
        // 构造一个最小 App 用于测试
        let prompt = format!(
            r#"你需要为当前项目生成一份指令文件，帮助后续的 AI 理解项目。

## 任务

请执行以下步骤并按结果生成内容：

### 步骤 1：了解项目

1. 读取项目的清单文件（Cargo.toml / package.json / pyproject.toml / go.mod 等）
2. 读取 README.md（如果存在）
3. 读取现有的指令文件（WHALE.md / AGENTS.md / CLAUDE.md，如果存在）
4. 列出项目的主要目录结构
5. 检查 CI 配置（.github/workflows/、Makefile、Dockerfile 等）
6. 了解项目的构建、测试、代码检查命令

### 步骤 2：分析架构

- 确定项目的关键模块和入口点
- 识别使用的框架和关键依赖

### 步骤 3：生成 {}

使用 `write_file` 工具将生成的指令写入 `{}`。

## 输出文件格式

```markdown
# 项目指令

## 项目类型

<项目类型>（<项目说明>）

**框架：** <框架列表>

### 命令
- 构建：`<构建命令>`
- 测试：`<测试命令>`
- 代码检查：`<代码检查命令>`
- 运行：`<运行命令>`

---

## 架构

### 关键模块

- **<模块名>**：<说明>

### 入口点

- `<入口路径>`

---

## 代码规范

- <规范说明>

---

## 额外信息

- <补充说明>

---

*由 &init 于 {} 生成*
```

## 已有内容处理

如果 `{}` 已经存在，先读取现有内容，然后：
1. 保留用户手写的部分（非自动生成的段落）
2. 补充缺失的信息（命令、架构、规范）
3. 更新过时的内容
4. 不要删除用户添加的自定义说明

## 注意事项

- 如果检测到 Rust 项目，检查 Cargo.toml 中的依赖推断框架和规范
- 如果检测到 Node.js 项目，检查 package.json 中的 scripts 和 devDependencies
- 对于 Python 项目，检查 pyproject.toml 中的构建系统和依赖
- 多模块项目（工作空间/monorepo）要列出各个模块的作用
- 不要写空泛的模板内容，每一条都要基于实际项目分析得出
- 不要输出无关的额外文件，只生成指令文件本身
- 如果已有 WHALE.md/AGENTS.md/CLAUDE.md，优先更新而不是覆盖
"#,
            "WHALE.md", "WHALE.md", "2025-06-01 12:00", "WHALE.md"
        );
        prompt
    }
}
