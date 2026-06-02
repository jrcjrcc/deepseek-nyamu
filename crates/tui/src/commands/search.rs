//! `&search <query>` — 快速文件内容搜索
//!
//! 在工作区中搜索文件内容，返回匹配最多的文件列表。
//! 尊重 .gitignore，只搜索常见源码文件。

use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fmt::Write;

use super::CommandResult;
use crate::tui::app::App;

/// 搜素的源码文件扩展名
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "go", "rb", "java", "kt", "scala", "c",
    "h", "cpp", "hpp", "cc", "hh", "md", "txt", "toml", "json", "yaml", "yml", "xml", "sh", "bash",
    "zsh", "ps1", "css", "scss", "less", "html", "vue", "svelte", "sql", "graphql", "proto",
    "lock",
];

/// 最大搜索结果数
const MAX_RESULTS: usize = 15;

/// 搜索入口
pub fn search(app: &App, arg: Option<&str>) -> CommandResult {
    let query = match arg.map(str::trim).filter(|s| !s.is_empty()) {
        Some(q) => q,
        None => return CommandResult::error("用法：&search <关键词>"),
    };

    let workspace = &app.workspace;
    let query_lower = query.to_lowercase();

    let mut matches: HashMap<String, usize> = HashMap::new();

    let walker = WalkBuilder::new(workspace)
        .hidden(false)
        .follow_links(false)
        .require_git(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // 检查扩展名
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };
        if !SOURCE_EXTENSIONS.contains(&ext) {
            continue;
        }

        // 跳过超大文件（> 1MB）
        if let Ok(meta) = path.metadata() {
            if meta.len() > 1_048_576 {
                continue;
            }
        }

        // 读文件内容
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // 统计匹配行数
        let count = content
            .lines()
            .filter(|line| line.to_lowercase().contains(&query_lower))
            .count();

        if count > 0 {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(path)
                .display()
                .to_string()
                .replace('\\', "/");
            matches.insert(rel, count);
        }
    }

    if matches.is_empty() {
        return CommandResult::message(format!("[搜索] 在工作区中未找到包含「{query}」的文件。"));
    }

    // 按匹配行数排序（降序）
    let mut sorted: Vec<(&String, &usize)> = matches.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let mut out = String::new();
    writeln!(
        out,
        "[搜索] 搜索「{query}」— 找到 {} 个文件（共 {} 个匹配）",
        sorted.len(),
        sorted.iter().map(|(_, c)| *c).sum::<usize>()
    )
    .unwrap();
    writeln!(out).unwrap();

    for (path, count) in sorted.iter().take(MAX_RESULTS) {
        writeln!(out, "  {count:>4}  {path}").unwrap();
    }

    if sorted.len() > MAX_RESULTS {
        writeln!(
            out,
            "  ... 还有 {} 个文件未显示",
            sorted.len() - MAX_RESULTS
        )
        .unwrap();
    }

    CommandResult::message(out)
}
