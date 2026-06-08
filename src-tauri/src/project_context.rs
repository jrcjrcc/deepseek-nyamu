//! Project context loading for DeepWhale.
//!
//! Loads project-specific instruction files and builds a project structure
//! overview (context pack) for injection into the system prompt.
//!
//! Instruction file priority:
//! 1. `AGENTS.md` - Cross-agent project instructions (canonical, highest)
//! 2. `WHALE.md` - Legacy CodeWhale-native instructions (deprecated fallback)
//! 3. `CLAUDE.md` - Claude-style instructions (compat)
//! 4. `.claude/instructions.md` - Hidden Claude instructions (compat)
//! 5. `.deepseek/instructions.md` - Hidden DeepSeek instructions (legacy)
//! 6. `.deepwhale/instructions.md` - DeepWhale-specific instructions
//!
//! Ported from CodeWhale crates/tui/src/project_context.rs

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use anyhow::{Context, Result};

// ─── Configuration ────────────────────────────────────────────────────

const PROJECT_INSTRUCTION_FILES: &[&str] = &[
    "AGENTS.md",
    "WHALE.md",
    "CLAUDE.md",
    ".claude/instructions.md",
    ".deepseek/instructions.md",
    ".deepwhale/instructions.md",
];

const MAX_CONTEXT_SIZE: usize = 100 * 1024; // 100KB
const PACK_README_MAX_CHARS: usize = 4_000;
const PACK_MAX_ENTRIES: usize = 220;
const PACK_MAX_DEPTH: usize = 4;

const PACK_IGNORED_DIRS: &[&str] = &[
    ".git", ".worktrees", "node_modules", ".venv", "venv",
    "__pycache__", "dist", "build", "target", ".idea", ".vscode",
    ".pytest_cache",
];

const PACK_IGNORED_EXTS: &[&str] = &[
    "7z", "avif", "db", "gif", "gz", "ico", "jpeg", "jpg",
    "log", "mov", "mp3", "mp4", "pdf", "png", "sqlite", "tar",
    "tgz", "wav", "webp", "zip",
];

const PACK_ALLOWED_HIDDEN: &[&str] = &[".github"];
const PACK_ALLOWED_HIDDEN_FILES: &[&str] = &[".editorconfig", ".gitattributes", ".gitignore"];

// ─── Types ────────────────────────────────────────────────────────────

/// Loaded project context.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub instructions: Option<String>,
    pub source_path: Option<PathBuf>,
    #[allow(dead_code)]
    pub project_root: PathBuf,
    #[allow(dead_code)]
    pub warnings: Vec<String>,
}

impl ProjectContext {
    pub fn empty(project_root: PathBuf) -> Self {
        Self { instructions: None, source_path: None, project_root, warnings: Vec::new() }
    }

    pub fn for_workspace(workspace: &Path) -> Self {
        let mut ctx = Self::empty(workspace.to_path_buf());

        // Walk up from workspace to find instruction files
        let mut current = Some(workspace.to_path_buf());
        while let Some(dir) = current {
            for filename in PROJECT_INSTRUCTION_FILES {
                let path = dir.join(filename);
                if path.exists() && ctx.instructions.is_none() {
                    match std::fs::read_to_string(&path) {
                        Ok(content) if !content.trim().is_empty() && content.len() <= MAX_CONTEXT_SIZE => {
                            ctx.instructions = Some(content.trim().to_string());
                            ctx.source_path = Some(path);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            // Walk up to parent
            current = dir.parent().map(|p| p.to_path_buf());
            if current.as_ref().map_or(true, |p| p.as_os_str().is_empty()) {
                break;
            }
        }

        ctx
    }

    #[allow(dead_code)]
    pub fn has_instructions(&self) -> bool {
        self.instructions.is_some()
    }

    #[allow(dead_code)]
    /// Render the project instructions as a `<project_instructions>` block.
    pub fn as_system_block(&self) -> Option<String> {
        self.instructions.as_ref().map(|content| {
            let source = self.source_path.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "project".to_string());
            format!(
                "<project_instructions source=\"{source}\">\n{content}\n</project_instructions>"
            )
        })
    }
}

// ─── Context Pack (directory tree scan) ──────────────────────────────

/// Build a project context pack for system prompt injection.
/// Mirrors the format used by CodeWhale's `<project_context_pack>` block.
pub fn build_context_pack(root: &Path) -> Option<String> {
    if !root.exists() || !root.is_dir() {
        return None;
    }

    let mut entries: Vec<String> = Vec::new();
    collect_pack_entries(root, root, 0, &mut entries);

    if entries.is_empty() {
        return None;
    }

    // Try to read README
    let readme = try_read_readme(root);

    // Identify key files
    let config_files: Vec<&str> = entries.iter()
        .filter(|e| is_config_file(e))
        .take(5)
        .map(|s| s.as_str())
        .collect();

    let source_files: Vec<&str> = entries.iter()
        .filter(|e| is_source_file(e))
        .take(15)
        .map(|s| s.as_str())
        .collect();

    // Count totals
    let dir_count = entries.iter().filter(|e| e.ends_with('/')).count();
    let _file_count = entries.len() - dir_count;

    Some(format!(
        "<project_context_pack>\n\
         <project_name>{name}</project_name>\n\
         <directory_tree>\n{tree}\n</directory_tree>\n\
         {readme_block}\
         <config_files>\n{configs}\n</config_files>\n\
         <key_source_files>\n{sources}\n</key_source_files>\n\
         <counts>\n\
           directory_entries: {total}\n\
           config_files: {cfg_count}\n\
           key_source_files: {src_count}\n\
         </counts>\n\
         </project_context_pack>",
        name = root.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
        tree = entries.join("\n"),
        readme_block = readme.map(|r| format!("<readme>{r}</readme>\n")).unwrap_or_default(),
        configs = if config_files.is_empty() { "none".to_string() } else { config_files.join("\n") },
        sources = if source_files.is_empty() { "none".to_string() } else { source_files.join("\n") },
        total = entries.len(),
        cfg_count = config_files.len(),
        src_count = source_files.len(),
    ))
}

fn collect_pack_entries(root: &Path, dir: &Path, depth: usize, out: &mut Vec<String>) {
    if depth > PACK_MAX_DEPTH || out.len() >= PACK_MAX_ENTRIES {
        return;
    }

    let mut queue = VecDeque::new();
    queue.push_back((dir.to_path_buf(), depth));

    while let Some((current_dir, current_depth)) = queue.pop_front() {
        if current_depth > PACK_MAX_DEPTH || out.len() >= PACK_MAX_ENTRIES {
            continue;
        }

        let Ok(read_dir) = std::fs::read_dir(&current_dir) else { continue };
        let mut children: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
        children.sort_by_key(|e| e.path());

        for entry in children {
            if out.len() >= PACK_MAX_ENTRIES { break; }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            let Ok(ftype) = entry.file_type() else { continue };

            if ftype.is_dir() {
                if should_ignore_dir(name) { continue; }
                if let Some(rel) = relative_slash_path(root, &path) {
                    out.push(format!("  DIR: {rel}"));
                    if current_depth < PACK_MAX_DEPTH {
                        queue.push_back((path, current_depth + 1));
                    }
                }
            } else if ftype.is_file() {
                if should_ignore_file(name) { continue; }
                if let Some(rel) = relative_slash_path(root, &path) {
                    let prefix = if is_config_file(&rel) { "  FILE: " }
                        else if is_source_file(&rel) { "  FILE: " }
                        else { "  FILE: " };
                    out.push(format!("{prefix}{rel}"));
                }
            }
        }
    }
}

fn should_ignore_dir(name: &str) -> bool {
    PACK_IGNORED_DIRS.contains(&name)
        || (name.starts_with('.') && !PACK_ALLOWED_HIDDEN.contains(&name))
}

fn should_ignore_file(name: &str) -> bool {
    if name.starts_with('.') && !PACK_ALLOWED_HIDDEN_FILES.contains(&name) {
        return true;
    }
    let Some((_, ext)) = name.rsplit_once('.') else { return false };
    PACK_IGNORED_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

fn relative_slash_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let path_str = relative.to_string_lossy().replace('\\', "/");
    if path_str.is_empty() || path_str == "." { None } else { Some(path_str) }
}

fn try_read_readme(root: &Path) -> Option<String> {
    for name in &["README.md", "README.txt", "README"] {
        let path = root.join(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let content = content.trim();
            if !content.is_empty() {
                let truncated: String = content.chars().take(PACK_README_MAX_CHARS).collect();
                return Some(truncated);
            }
        }
    }
    None
}

fn is_config_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with("config.toml") || lower.ends_with("config.yml") || lower.ends_with("config.yaml")
        || lower.ends_with("config.json") || lower.ends_with("cargo.toml") || lower.ends_with("package.json")
        || lower.ends_with("pyproject.toml") || lower.ends_with("dockerfile") || lower.ends_with("docker-compose.yml")
        || lower.ends_with(".env.example") || lower.ends_with("compose.yml") || lower.ends_with("makefile")
        || lower.ends_with("tsconfig.json") || lower.ends_with(".editorconfig")
}

fn is_source_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".rs") || lower.ends_with(".py") || lower.ends_with(".ts") || lower.ends_with(".tsx")
        || lower.ends_with(".js") || lower.ends_with(".jsx") || lower.ends_with(".go") || lower.ends_with(".java")
        || lower.ends_with(".c") || lower.ends_with(".cpp") || lower.ends_with(".h") || lower.ends_with(".hpp")
        || lower.ends_with(".rb") || lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".ps1")
        || lower.ends_with(".kt") || lower.ends_with(".swift") || lower.ends_with(".css") || lower.ends_with(".scss")
        || lower.ends_with(".sql") || lower.ends_with(".r") || lower.ends_with(".mjs")
}

/// Build the complete project context block for system prompt injection.
pub fn build_full_project_block(workspace: &Path) -> String {
    let ctx = ProjectContext::for_workspace(workspace);
    let mut blocks = Vec::new();

    // Instructions block
    if let Some(block) = ctx.as_system_block() {
        blocks.push(block);
    }

    // Context pack
    if let Some(pack) = build_context_pack(workspace) {
        blocks.push(pack);
    }

    blocks.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_agents_md_in_workspace() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("AGENTS.md"), "test instructions").unwrap();
        let ctx = ProjectContext::for_workspace(tmp.path());
        assert!(ctx.has_instructions());
        assert!(ctx.instructions.unwrap().contains("test instructions"));
    }

    #[test]
    fn finds_claude_md_fallback() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("CLAUDE.md"), "claude rules").unwrap();
        let ctx = ProjectContext::for_workspace(tmp.path());
        assert!(ctx.has_instructions());
        assert!(ctx.instructions.unwrap().contains("claude rules"));
    }

    #[test]
    fn returns_empty_when_no_instruction_files() {
        let tmp = TempDir::new().unwrap();
        let ctx = ProjectContext::for_workspace(tmp.path());
        assert!(!ctx.has_instructions());
    }

    #[test]
    fn builds_context_pack_shows_directory_structure() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        let pack = build_context_pack(tmp.path());
        assert!(pack.is_some());
        let pack = pack.unwrap();
        assert!(pack.contains("<project_context_pack>"));
        assert!(pack.contains("main.rs"));
    }

    #[test]
    fn full_block_combines_instructions_and_pack() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("AGENTS.md"), "rules").unwrap();
        fs::write(tmp.path().join("src").join("lib.rs"), "pub fn foo() {}").unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        let block = build_full_project_block(tmp.path());
        assert!(block.contains("<project_instructions"));
        assert!(block.contains("<project_context_pack"));
    }
}
