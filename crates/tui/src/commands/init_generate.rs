//! `/init -i` 文件生成模块
//!
//! 根据扫描结果和用户偏好，渲染 WHALE.md / AGENTS.md / CLAUDE.md。
//! 生成的是实质性内容，不是模板骨架。

use std::fmt::Write;
use std::path::Path;

use super::init_survey::SurveyResult;
use super::init_wizard::DetailLevel;

/// 生成指令文件
///
/// # 参数
/// - `survey`: 项目扫描结果
/// - `output_path`: 输出路径
/// - `detail`: 详细程度
/// - `conventions`: 用户补充的代码规范
pub fn generate(
    survey: &SurveyResult,
    output_path: &Path,
    detail: DetailLevel,
    conventions: &[String],
) -> std::io::Result<()> {
    let content = render(survey, detail, conventions);
    std::fs::write(output_path, content)
}

/// 渲染文件内容
fn render(survey: &SurveyResult, detail: DetailLevel, conventions: &[String]) -> String {
    let mut doc = String::new();

    // ── 头部 ───────────────────────────────────────────────────────
    writeln!(doc, "# 项目指令").unwrap();
    writeln!(doc).unwrap();
    writeln!(
        doc,
        "> 由 `/init -i` 自动生成，可编辑修改。"
    )
    .unwrap();
    writeln!(doc).unwrap();

    // ── 项目类型 + 命令 ───────────────────────────────────────────
    writeln!(doc, "## 项目类型").unwrap();
    writeln!(doc).unwrap();

    if !survey.project_type_detail.is_empty() {
        writeln!(
            doc,
            "{typ}（{detail}）",
            typ = survey.project_type,
            detail = survey.project_type_detail
        )
        .unwrap();
    } else {
        writeln!(doc, "{}", survey.project_type).unwrap();
    }
    writeln!(doc).unwrap();

    // 框架
    if !survey.frameworks.is_empty() {
        writeln!(doc, "**框架：** {}。", survey.frameworks.join("、")).unwrap();
        writeln!(doc).unwrap();
    }

    // 命令（无论哪种详细程度都显示）
    writeln!(doc, "### 命令").unwrap();
    writeln!(doc).unwrap();

    let all_commands = [
        ("构建", &survey.build_commands),
        ("测试", &survey.test_commands),
        ("代码检查", &survey.lint_commands),
        ("运行", &survey.run_commands),
    ];

    for (label, cmds) in &all_commands {
        if cmds.is_empty() {
            continue;
        }
        let bullets: Vec<String> = cmds.iter().map(|c| format!("- `{c}`")).collect();
        writeln!(doc, "{label}：").unwrap();
        for b in &bullets {
            writeln!(doc, "{b}").unwrap();
        }
        writeln!(doc).unwrap();
    }

    // ── 架构（Normal / Detailed） ─────────────────────────────────
    if detail != DetailLevel::Brief {
        writeln!(doc, "---").unwrap();
        writeln!(doc).unwrap();
        writeln!(doc, "## 架构").unwrap();
        writeln!(doc).unwrap();

        if !survey.key_modules.is_empty() {
            writeln!(doc, "### 关键模块").unwrap();
            writeln!(doc).unwrap();
            for (name, role) in &survey.key_modules {
                writeln!(doc, "- **{name}**：{role}").unwrap();
            }
            writeln!(doc).unwrap();
        }

        if !survey.entry_points.is_empty() {
            writeln!(doc, "### 入口点").unwrap();
            writeln!(doc).unwrap();
            for ep in &survey.entry_points {
                writeln!(doc, "- `{ep}`").unwrap();
            }
            writeln!(doc).unwrap();
        }
    }

    // ── 代码规范（Normal / Detailed） ──────────────────────────────
    if detail != DetailLevel::Brief {
        let has_custom = !conventions.is_empty();
        let has_auto = !survey.conventions_to_note.is_empty();

        if has_custom || has_auto {
            writeln!(doc, "---").unwrap();
            writeln!(doc).unwrap();
            writeln!(doc, "## 代码规范").unwrap();
            writeln!(doc).unwrap();
        }

        for c in survey.conventions_to_note.iter().chain(conventions.iter()) {
            writeln!(doc, "- {c}").unwrap();
        }
        if has_custom || has_auto {
            writeln!(doc).unwrap();
        }
    }

    // ── 额外说明（Detailed） ──────────────────────────────────────
    if detail == DetailLevel::Detailed {
        if !survey.notes.is_empty() {
            writeln!(doc, "---").unwrap();
            writeln!(doc).unwrap();
            writeln!(doc, "## 额外信息").unwrap();
            writeln!(doc).unwrap();
            for note in &survey.notes {
                writeln!(doc, "- {note}").unwrap();
            }
            writeln!(doc).unwrap();
        }

        if survey.has_readme {
            writeln!(doc, "- 参考 `README.md` 了解项目概述。").unwrap();
            writeln!(doc).unwrap();
        }
    }

    // ── 占位引导 ──────────────────────────────────────────────────
    writeln!(doc, "---").unwrap();
    writeln!(doc).unwrap();
    writeln!(doc, "## 需要你补充的").unwrap();
    writeln!(doc).unwrap();
    writeln!(doc, "- 如果自动检测的命令不准，请修正上面的命令。").unwrap();
    if survey.entry_points.is_empty() {
        writeln!(doc, "- 填写入口点（程序从哪里启动）。").unwrap();
    }
    if survey.key_modules.is_empty() {
        writeln!(doc, "- 补充关键模块的说明。").unwrap();
    }
    writeln!(doc).unwrap();
    writeln!(
        doc,
        "---\n*由 `/init -i` 于 {} 生成*",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    )
    .unwrap();

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_survey() -> SurveyResult {
        SurveyResult {
            project_type: "Rust".to_string(),
            project_type_detail: "my-project, 14 个 crate 的工作空间".to_string(),
            build_commands: vec!["cargo build".to_string()],
            test_commands: vec![
                "cargo test".to_string(),
                "cargo test -p some-crate".to_string(),
            ],
            lint_commands: vec!["cargo clippy".to_string(), "cargo fmt".to_string()],
            run_commands: vec!["cargo run".to_string()],
            frameworks: vec![
                "Ratatui (TUI)".to_string(),
                "Tokio".to_string(),
            ],
            key_modules: vec![
                ("src".to_string(), "核心源码".to_string()),
                ("crates/cli".to_string(), "CLI 接口".to_string()),
                ("crates/tui".to_string(), "终端 UI".to_string()),
            ],
            entry_points: vec!["crates/cli/src/lib.rs".to_string()],
            conventions_to_note: vec![
                "Rust 2024 edition".to_string(),
                "使用 anyhow::Result 处理错误".to_string(),
                "Tokio 异步运行时".to_string(),
                "serde 用于序列化".to_string(),
            ],
            notes: vec![
                "GitHub Actions 配置（3 个工作流）".to_string(),
                "Docker 容器化部署".to_string(),
            ],
            has_readme: true,
        }
    }

    #[test]
    fn test_generate_brief() {
        let survey = make_survey();
        let output = render(&survey, DetailLevel::Brief, &[]);
        assert!(output.contains("项目类型"));
        assert!(output.contains("cargo build"));
        assert!(output.contains("cargo test"));
        assert!(!output.contains("架构"));
        assert!(!output.contains("代码规范"));
        assert!(!output.contains("入口"));
    }

    #[test]
    fn test_generate_normal() {
        let survey = make_survey();
        let output = render(&survey, DetailLevel::Normal, &[]);
        assert!(output.contains("架构"));
        assert!(output.contains("代码规范"));
        assert!(output.contains("关键模块"));
        assert!(output.contains("cargo clippy"));
        assert!(!output.contains("额外信息"));
    }

    #[test]
    fn test_generate_detailed() {
        let survey = make_survey();
        let output = render(&survey, DetailLevel::Detailed, &["自定义规范：xxx".to_string()]);
        assert!(output.contains("额外信息"));
        assert!(output.contains("GitHub Actions"));
        assert!(output.contains("Docker"));
        assert!(output.contains("自定义规范"));
        assert!(output.contains("README.md"));
    }

    #[test]
    fn test_generate_includes_user_conventions() {
        let survey = make_survey();
        let output = render(&survey, DetailLevel::Normal, &["缩进用两个空格".to_string()]);
        assert!(output.contains("缩进用两个空格"));
    }

    #[test]
    fn test_generate_file_is_written() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("WHALE.md");
        let survey = make_survey();
        generate(&survey, &path, DetailLevel::Normal, &[]).unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("项目类型"));
    }
}
