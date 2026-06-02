//! `&bak` / `&rmb` — backup and restore code file backups.

use std::fs;
use std::path::Path;

use super::CommandResult;
use crate::tui::app::App;

/// Source file extensions that get backed up by `&bak`.
const CODE_EXTENSIONS: &[&str] = &[
    "c", "h", "cpp", "hpp", "cc", "cxx", "hh",
    "js", "jsx", "mjs", "cjs",
    "ts", "tsx", "mts", "cts",
    "css", "scss", "sass", "less",
    "rs", "py", "java", "go", "rb", "php",
    "swift", "kt", "scala", "pl", "pm",
    "sh", "bash", "zsh", "lua",
    "toml", "json", "yaml", "yml", "xml",
    "html", "htm", "vue", "svelte", "md",
    "tex", "r", "m", "mm",
    "s", "asm", "zig", "ex", "exs",
    "gradle", "cmake", "mk",
];

/// `&bak` — recursively copy all code files as `.bak` backups.
///
/// For each code file found under the workspace, creates `<file>.bak` if it
/// does not already exist.  Skips files that already have a `.bak` sibling.
pub fn bak(app: &mut App) -> CommandResult {
    let workspace = app.workspace.clone();
    if workspace.as_os_str().is_empty() || !workspace.exists() {
        return CommandResult::error("工作区目录不可用，无法创建备份。请先进入项目目录。");
    }

    let mut count = 0;
    let mut errors = Vec::new();

    visit_dirs(&workspace, &mut |path| {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if CODE_EXTENSIONS.contains(&ext) {
                let bak_path = path.with_extension(format!("{ext}.bak"));
                if !bak_path.exists() {
                    match fs::copy(path, &bak_path) {
                        Ok(_) => count += 1,
                        Err(e) => errors.push(format!("{}: {e}", path.display())),
                    }
                }
            }
        }
    });

    let mut msg = format!("✅ 已备份 {count} 个代码文件（已存在的 .bak 跳过）");
    if !errors.is_empty() {
        let err_list = errors.join("\n  ");
        msg.push_str(&format!("\n⚠️  以下文件备份失败：\n  {err_list}"));
    }
    CommandResult::message(msg)
}

/// `&rmb` — recursively remove all `.bak` files.
///
/// Scans the workspace tree and deletes every `<name>.<ext>.bak` file found.
pub fn rmb(app: &mut App) -> CommandResult {
    let workspace = app.workspace.clone();
    if workspace.as_os_str().is_empty() || !workspace.exists() {
        return CommandResult::error("工作区目录不可用。请先进入项目目录。");
    }

    let mut count = 0;
    let mut errors = Vec::new();

    visit_dirs(&workspace, &mut |path| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".bak") {
                // Only delete files matching <name>.<ext>.bak pattern
                let stem = name.strip_suffix(".bak").unwrap_or(name);
                if stem.contains('.') {
                    match fs::remove_file(path) {
                        Ok(_) => count += 1,
                        Err(e) => errors.push(format!("{}: {e}", path.display())),
                    }
                }
            }
        }
    });

    let mut msg = format!("🧹 已删除 {count} 个 .bak 备份文件");
    if !errors.is_empty() {
        let err_list = errors.join("\n  ");
        msg.push_str(&format!("\n⚠️  以下文件删除失败：\n  {err_list}"));
    }
    CommandResult::message(msg)
}

/// Recursively walk `dir` and call `f` for every regular file.
fn visit_dirs(dir: &Path, f: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Skip hidden dirs (node_modules, .git, .deepseek, target, …)
        if path.is_dir() {
            let keep = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('.') && n != "node_modules" && n != "target")
                .unwrap_or(false);
            if keep {
                visit_dirs(&path, f);
            }
        } else if path.is_file() {
            f(&path);
        }
    }
}
