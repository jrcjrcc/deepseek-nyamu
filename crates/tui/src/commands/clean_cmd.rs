//! `&cleanproject` — 清理项目临时文件
//!
//! 扫描并清理开发过程中产生的临时文件。
//! 安全模式：默认只列出不删除。
//! - `&cleanproject`          → 列出可清理的文件
//! - `&cleanproject --run`    → 列出并删除

use std::fmt::Write;

use ignore::WalkBuilder;

use super::CommandResult;
use crate::tui::app::App;

/// 临时文件扩展名
const TEMP_EXTENSIONS: &[&str] = &["log", "tmp", "bak", "swp", "swo", "~"];

/// 保护的目录
const PROTECTED_DIRS: &[&str] = &[
    ".git",
    ".codewhale",
    ".deepseek",
    ".claude",
    "node_modules",
    "vendor",
    "target",
    ".venv",
    "__pycache__",
    ".idea",
    ".vscode",
];

/// 清理入口
pub fn cleanproject(app: &App, arg: Option<&str>) -> CommandResult {
    let workspace = &app.workspace;
    let run_mode = arg
        .map(str::trim)
        .map(|s| s.eq_ignore_ascii_case("--run") || s.eq_ignore_ascii_case("-r"))
        .unwrap_or(false);

    let mut found: Vec<String> = Vec::new();
    let mut total_size: u64 = 0;

    let walker = WalkBuilder::new(workspace)
        .hidden(false)
        .follow_links(false)
        .require_git(true)
        .max_depth(Some(8))
        .build();

    'outer: for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // 跳过保护目录
        if let Ok(rel) = path.strip_prefix(workspace) {
            let components: Vec<_> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
            for comp in &components {
                let c = comp.as_str();
                if PROTECTED_DIRS.contains(&c) {
                    continue 'outer;
                }
            }
        }

        // 检查扩展名
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let is_temp = TEMP_EXTENSIONS.contains(&ext.as_str());
        let ends_with_tilde = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with('~'))
            .unwrap_or(false);

        if !is_temp && !ends_with_tilde {
            continue;
        }

        let rel = path
            .strip_prefix(workspace)
            .unwrap_or(path)
            .display()
            .to_string()
            .replace('\\', "/");
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        found.push(rel);
        total_size += size;
    }

    if found.is_empty() {
        return CommandResult::message("[清理] 未发现临时文件，项目很干净。");
    }

    // 排序
    found.sort();

    let mut out = String::new();
    writeln!(
        out,
        "[清理] 发现 {} 个临时文件（共 {}）",
        found.len(),
        format_size(total_size)
    )
    .unwrap();
    writeln!(out).unwrap();

    for f in &found {
        writeln!(out, "  {f}").unwrap();
    }

    if run_mode {
        // 执行删除
        let mut deleted = 0usize;
        let mut errors = 0usize;
        for f in &found {
            let full = workspace.join(f.replace('/', "\\"));
            if std::fs::remove_file(&full).is_ok() {
                deleted += 1;
            } else {
                errors += 1;
            }
        }
        writeln!(out).unwrap();
        writeln!(out, "[完成] 已删除 {deleted} 个文件").unwrap();
        if errors > 0 {
            writeln!(out, "[警告]️  删除失败 {errors} 个").unwrap();
        }
    } else {
        writeln!(out).unwrap();
        writeln!(out, "使用 `&cleanproject --run` 执行删除。").unwrap();
    }

    CommandResult::message(out)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
