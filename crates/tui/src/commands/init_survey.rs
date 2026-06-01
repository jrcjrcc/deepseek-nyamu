//! `/init -i` 项目扫描模块
//!
//! 读取项目文件，返回结构化的项目信息。
//! 当前为同步版本（直接读文件），未来可替换为子代理版本。

use std::path::Path;

/// 项目扫描结果
#[derive(Debug, Clone)]
pub struct SurveyResult {
    /// 项目类型（Rust / Node.js / Python / Go / other）
    pub project_type: String,
    /// 项目类型说明（如 "14 个 crate 的工作空间"）
    pub project_type_detail: String,
    /// 构建命令
    pub build_commands: Vec<String>,
    /// 测试命令
    pub test_commands: Vec<String>,
    /// 代码检查命令
    pub lint_commands: Vec<String>,
    /// 运行命令
    pub run_commands: Vec<String>,
    /// 框架列表
    pub frameworks: Vec<String>,
    /// 关键模块
    pub key_modules: Vec<(String, String)>, // (路径, 说明)
    /// 入口点
    pub entry_points: Vec<String>,
    /// 需要记录的代码规范
    pub conventions_to_note: Vec<String>,
    /// 发现的其他有用信息
    pub notes: Vec<String>,
    /// 文件是否存在
    pub has_readme: bool,
}

/// 扫描项目，返回结构化信息
pub fn survey_project(workspace: &Path) -> SurveyResult {
    let mut result = SurveyResult {
        project_type: String::new(),
        project_type_detail: String::new(),
        build_commands: Vec::new(),
        test_commands: Vec::new(),
        lint_commands: Vec::new(),
        run_commands: Vec::new(),
        frameworks: Vec::new(),
        key_modules: Vec::new(),
        entry_points: Vec::new(),
        conventions_to_note: Vec::new(),
        notes: Vec::new(),
        has_readme: false,
    };

    // ── README ─────────────────────────────────────────────────────
    result.has_readme = workspace.join("README.md").exists();

    // ── 检测项目类型 ──────────────────────────────────────────────
    detect_project_type(workspace, &mut result);

    // ── 检测框架 ──────────────────────────────────────────────────
    detect_frameworks(workspace, &mut result);

    // ── 扫描目录结构（关键模块） ──────────────────────────────────
    scan_directories(workspace, &mut result);

    // ── 检测 CI ───────────────────────────────────────────────────
    detect_ci(workspace, &mut result);

    result
}

/// 项目类型检测
fn detect_project_type(workspace: &Path, result: &mut SurveyResult) {
    if workspace.join("Cargo.toml").exists() {
        result.project_type = "Rust".to_string();
        result.build_commands.push("cargo build".to_string());
        result.test_commands.push("cargo test".to_string());
        result.lint_commands.push("cargo clippy".to_string());
        result.run_commands.push("cargo run".to_string());

        // 检测工作空间
        if let Ok(content) = std::fs::read_to_string(workspace.join("Cargo.toml")) {
            if content.contains("[workspace]") {
                let crate_count = content
                    .lines()
                    .filter(|l| l.trim().starts_with('"') || l.trim().starts_with('\''))
                    .count();
                if crate_count > 0 {
                    result
                        .project_type_detail
                        .push_str(&format!("{crate_count} 个 crate 的工作空间"));
                }
            }
            // 提取项目名
            for line in content.lines() {
                let t = line.trim();
                if let Some(name) = t
                    .strip_prefix("name = \"")
                    .and_then(|s| s.split('"').next())
                {
                    if result.project_type_detail.is_empty() {
                        result.project_type_detail = name.to_string();
                    }
                    break;
                }
            }
        }
    } else if workspace.join("package.json").exists() {
        result.project_type = "Node.js".to_string();
        result.build_commands.push("npm run build".to_string());
        result.test_commands.push("npm test".to_string());
        result.run_commands.push("npm start".to_string());

        if let Ok(content) = std::fs::read_to_string(workspace.join("package.json")) {
            // 检测项目名
            if let Some(name) = content
                .lines()
                .find(|l| l.trim().starts_with("\"name\""))
                .and_then(|l| {
                    l.split(':')
                        .nth(1)
                        .map(|s| s.trim().trim_matches(',').trim_matches('"').to_string())
                })
            {
                if !name.is_empty() && name != "name" {
                    result.project_type_detail = name;
                }
            }
        }
    } else if workspace.join("pyproject.toml").exists() || workspace.join("setup.py").exists() {
        result.project_type = "Python".to_string();
        result.test_commands.push("pytest".to_string());
        result.lint_commands.push("ruff check .".to_string());
        if workspace.join("pyproject.toml").exists() {
            result.build_commands.push("pip install -e .".to_string());
        }
    } else if workspace.join("go.mod").exists() {
        result.project_type = "Go".to_string();
        result.build_commands.push("go build".to_string());
        result.test_commands.push("go test ./...".to_string());
        result.run_commands.push("go run .".to_string());
        result.lint_commands.push("go fmt ./...".to_string());
    } else {
        result.project_type = "other".to_string();
        result.notes.push("未识别项目类型，请在生成后补充命令".to_string());
    }
}

/// 框架检测
fn detect_frameworks(workspace: &Path, result: &mut SurveyResult) {
    // 前端框架
    if workspace.join("next.config.js").exists() || workspace.join("next.config.ts").exists() {
        result.frameworks.push("Next.js".to_string());
    } else if workspace.join("vite.config.js").exists()
        || workspace.join("vite.config.ts").exists()
    {
        result.frameworks.push("Vite".to_string());
    } else if workspace.join("astro.config.mjs").exists()
        || workspace.join("astro.config.ts").exists()
    {
        result.frameworks.push("Astro".to_string());
    }

    // Rust web 框架
    if let Ok(content) = std::fs::read_to_string(workspace.join("Cargo.toml")) {
        if content.contains("axum") {
            result.frameworks.push("Axum".to_string());
        }
        if content.contains("actix") {
            result.frameworks.push("Actix".to_string());
        }
        if content.contains("leptos") {
            result.frameworks.push("Leptos".to_string());
        }
        if content.contains("yew") {
            result.frameworks.push("Yew".to_string());
        }
        if content.contains("dioxus") {
            result.frameworks.push("Dioxus".to_string());
        }
        if content.contains("ratatui") {
            result.frameworks.push("Ratatui (TUI)".to_string());
        }
        if content.contains("serde") {
            result.conventions_to_note.push("serde 用于序列化/反序列化".to_string());
        }
        if content.contains("anyhow") {
            result.conventions_to_note.push("使用 anyhow::Result 处理错误".to_string());
        }
        if content.contains("thiserror") {
            result.conventions_to_note.push("使用 thiserror 定义错误类型".to_string());
        }
        if content.contains("tokio") {
            result.conventions_to_note.push("Tokio 异步运行时".to_string());
        }
        if content.contains("tracing") {
            result.conventions_to_note.push("使用 tracing 进行日志".to_string());
        }
        if content.contains("clap") {
            result.conventions_to_note.push("使用 clap 处理 CLI 参数".to_string());
        }
        if content.contains("edition = \"2024\"") {
            result.conventions_to_note.push("Rust 2024 edition".to_string());
        }
    }
}

/// 扫描目录结构，识别关键模块
fn scan_directories(workspace: &Path, result: &mut SurveyResult) {
    // 只读顶级目录，最多 6 个关键模块
    let mut dirs: Vec<(String, String)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(workspace) {
        for entry in entries.flatten() {
            if let Ok(ftype) = entry.file_type() {
                if ftype.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // 跳过隐藏目录和常见无关目录
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == "vendor"
                    {
                        continue;
                    }
                    dirs.push((name.clone(), detect_dir_role(&name, workspace)));
                }
            }
        }
    }

    // 排序：src/ 优先，其余按名字排序
    dirs.sort_by(|a, b| {
        let a_prio = if a.0 == "src" { 0 } else { 1 };
        let b_prio = if b.0 == "src" { 0 } else { 1 };
        a_prio.cmp(&b_prio).then(a.0.cmp(&b.0))
    });

    for (name, role) in dirs.into_iter().take(6) {
        result.key_modules.push((name, role));
    }
}

/// 根据目录名推断作用
fn detect_dir_role(name: &str, workspace: &Path) -> String {
    match name {
        "src" => "核心源码".to_string(),
        "crates" => "Rust 工作空间 crate".to_string(),
        "lib" | "library" => "库代码".to_string(),
        "bin" | "cmd" => "可执行程序入口".to_string(),
        "cli" => "CLI 接口".to_string(),
        "api" | "apis" => "API 接口".to_string(),
        "web" | "frontend" | "ui" | "app" => "前端/UI".to_string(),
        "server" | "backend" => "后端服务".to_string(),
        "core" => "核心逻辑".to_string(),
        "config" | "configuration" => "配置管理".to_string(),
        "tools" | "utils" | "util" | "helpers" => "工具/辅助".to_string(),
        "models" | "model" | "entities" | "entity" => "数据模型".to_string(),
        "db" | "database" | "migrations" | "schema" => "数据库".to_string(),
        "tests" | "test" | "spec" | "specs" => "测试".to_string(),
        "docs" | "doc" | "documentation" => "文档".to_string(),
        "scripts" | "script" => "脚本".to_string(),
        "docker" | "deploy" | "deployment" => "部署/Docker".to_string(),
        "examples" | "example" => "示例代码".to_string(),
        "plugins" | "plugin" | "extensions" | "extension" => "插件/扩展".to_string(),
        "hooks" => "钩子".to_string(),
        "mcp" => "MCP 服务".to_string(),
        "proto" | "protocol" => "协议定义".to_string(),
        "types" | "type" => "类型定义".to_string(),
        "shims" => "兼容垫片".to_string(),
        "vendor" => "第三方依赖".to_string(),
        "native-ts" | "native" => "原生绑定".to_string(),
        _ => {
            // 检查是否有 Cargo.toml 来判断是否是 Rust crate
            if workspace.join(name).join("Cargo.toml").exists() {
                "Rust crate".to_string()
            } else if workspace.join(name).join("package.json").exists() {
                "Node 包".to_string()
            } else {
                "模块目录".to_string()
            }
        }
    }
}

/// 检测 CI 配置
fn detect_ci(workspace: &Path, result: &mut SurveyResult) {
    let ci_dir = workspace.join(".github").join("workflows");
    if ci_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&ci_dir) {
            let count = entries.flatten().count();
            if count > 0 {
                result.notes.push(format!("GitHub Actions 配置（{count} 个工作流）"));
            }
        }
    }
    if workspace.join(".gitlab-ci.yml").exists() {
        result.notes.push("GitLab CI 配置".to_string());
    }
    if workspace.join("Makefile").exists() {
        result.notes.push("Makefile 构建系统".to_string());
    }
    if workspace.join("Dockerfile").exists() {
        result.notes.push("Docker 容器化部署".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_workspace() -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
edition = "2024"

[dependencies]
tokio = "1"
serde = "1"
anyhow = "1"
tracing = "0.1"
clap = "4"
thiserror = "1"

[workspace]
members = ["crates/core", "crates/cli"]
"#,
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::create_dir_all(tmp.path().join("crates")).unwrap();
        std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("tests")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Test").unwrap();
        std::fs::write(tmp.path().join("Makefile"), "all:\n\techo hi").unwrap();
        std::fs::write(
            tmp.path().join(".github/workflows/ci.yml"),
            "name: CI",
        )
        .unwrap();
        tmp
    }

    #[test]
    fn test_detects_rust_project() {
        let tmp = create_test_workspace();
        let result = survey_project(tmp.path());
        assert_eq!(result.project_type, "Rust");
        assert!(result.build_commands.contains(&"cargo build".to_string()));
        assert!(result.test_commands.contains(&"cargo test".to_string()));
        assert!(result.lint_commands.contains(&"cargo clippy".to_string()));
    }

    #[test]
    fn test_detects_conventions() {
        let tmp = create_test_workspace();
        let result = survey_project(tmp.path());
        assert!(result.conventions_to_note.iter().any(|c| c.contains("anyhow")));
        assert!(result.conventions_to_note.iter().any(|c| c.contains("serde")));
        assert!(result.conventions_to_note.iter().any(|c| c.contains("Tokio")));
        assert!(result.conventions_to_note.iter().any(|c| c.contains("clap")));
        assert!(result.conventions_to_note.iter().any(|c| c.contains("2024 edition")));
    }

    #[test]
    fn test_detects_readme_and_ci() {
        let tmp = create_test_workspace();
        let result = survey_project(tmp.path());
        assert!(result.has_readme);
        assert!(result.notes.iter().any(|n| n.contains("GitHub Actions")));
        assert!(result.notes.iter().any(|n| n.contains("Makefile")));
    }

    #[test]
    fn test_detects_node_project() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("package.json"),
            r#"{"name": "my-app", "scripts": {"build": "tsc"}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let result = survey_project(tmp.path());
        assert_eq!(result.project_type, "Node.js");
        assert!(result.build_commands.contains(&"npm run build".to_string()));
        assert!(result.test_commands.contains(&"npm test".to_string()));
    }

    #[test]
    fn test_detects_python_project() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("pyproject.toml"), "[project]\nname = \"py-app\"")
            .unwrap();
        let result = survey_project(tmp.path());
        assert_eq!(result.project_type, "Python");
        assert!(result.test_commands.contains(&"pytest".to_string()));
    }

    #[test]
    fn test_detects_go_project() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("go.mod"), "module test").unwrap();
        let result = survey_project(tmp.path());
        assert_eq!(result.project_type, "Go");
    }

    #[test]
    fn test_unknown_project() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("some_file.txt"), "hi").unwrap();
        let result = survey_project(tmp.path());
        assert_eq!(result.project_type, "other");
    }
}
