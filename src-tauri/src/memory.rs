//! 持久化用户记忆系统
//!
//! 管理一个记忆文件（~/.deepseek/memory.md），存储用户的声明性事实
//! （偏好、项目约定等）。记忆内容在每次对话时注入到系统提示词中，
//! 使模型能够感知持久化的用户偏好。
//!
//! 通过 DEEPSEEK_MEMORY 环境变量控制启用/禁用，
//! 通过 DEEPSEEK_MEMORY_PATH 自定义记忆文件路径。
//!
//! Ported from CodeWhale crates/tui/src/commands/memory.rs
//!
//! Persistent user-memory system.
//!
//! Manages a memory file that stores declarative facts about the user's
//! preferences and project conventions. Memory content is injected into
//! the system prompt so the model is aware of durable user preferences.
//!
//! Ported from CodeWhale crates/tui/src/commands/memory.rs

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Default memory file path.
fn default_memory_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".deepseek").join("memory.md")
}

/// Resolve the memory file path: check env var, then config, then default.
pub fn resolve_global_memory_path(config_memory_path: Option<PathBuf>) -> PathBuf {
    if let Some(path) = config_memory_path {
        return path;
    }
    std::env::var("DEEPSEEK_MEMORY_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(default_memory_path)
}

/// Check if user memory is enabled.
pub fn memory_enabled() -> bool {
    match std::env::var("DEEPSEEK_MEMORY").as_deref() {
        Ok("off" | "false" | "0") => false,
        _ => true,
    }
}

/// Read the current memory file content.
pub fn read_memory(path: &PathBuf) -> Result<String> {
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read memory file: {}", path.display()))
}

/// Write new content to the memory file.
pub fn write_memory(path: &PathBuf, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create memory dir: {}", parent.display()))?;
    }
    std::fs::write(path, content)
        .with_context(|| format!("failed to write memory file: {}", path.display()))
}

/// Append a timestamped bullet point to the memory file.
pub fn quick_capture(path: &PathBuf, note: &str) -> Result<String> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let line = format!("- [{timestamp}] {note}\n");
    let mut existing = read_memory(path).unwrap_or_default();
    if !existing.trim().is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&line);
    write_memory(path, &existing)?;
    Ok(line.trim().to_string())
}

/// Build the layered memory block for system prompt injection.
pub fn build_layered_memory_block(
    global_path: &PathBuf,
    _workspace: Option<&Path>,
    _session_summary: Option<&str>,
) -> String {
    if !memory_enabled() {
        return String::new();
    }
    let content = read_memory(global_path).unwrap_or_default();
    if content.trim().is_empty() {
        return String::new();
    }
    format!(
        "## User Memory\n\n{}\n\n{}",
        content.trim(),
        include_str!("../../vendor/nyamu/prompts/memory_guidance.md")
    )
}

#[allow(dead_code)]
/// Build the memory block (simple wrapper).
pub fn build_memory_block(path: &PathBuf) -> String {
    build_layered_memory_block(path, None, None)
}

/// CLI memory subcommand handler.
pub fn cmd_memory(args: &[String]) -> Result<String> {
    let path = resolve_global_memory_path(None);
    let sub = args.first().map(|s| s.as_str()).unwrap_or("show");

    match sub {
        "show" | "" => {
            if !memory_enabled() {
                return Ok("Memory is disabled. Set DEEPSEEK_MEMORY=on or remove the env var to enable.".to_string());
            }
            let body = match std::fs::read_to_string(&path) {
                Ok(text) if text.trim().is_empty() => format!(
                    "{}\n(empty \u{2014} use `# note` from the composer to add entries)",
                    path.display()
                ),
                Ok(text) => format!("{}\n\n{}", path.display(), text.trim_end()),
                Err(_) => format!("{}\n(file does not exist yet)", path.display()),
            };
            Ok(body)
        }
        "path" => Ok(path.display().to_string()),
        "clear" => {
            write_memory(&path, "")?;
            Ok(format!("Memory cleared: {}", path.display()))
        }
        "add" => {
            let note = args.get(1).map(|s| s.as_str()).unwrap_or("");
            if note.is_empty() {
                Ok("Usage: memory add <note>".to_string())
            } else {
                let line = quick_capture(&path, note)?;
                Ok(format!("Added: {line}"))
            }
        }
        "help" | _ => Ok(format!(
            "Usage: memory [show|path|clear|add <note>|help]\n\n\
             Current path: {}\n\n\
             Subcommands:\n\
               memory              Show memory file path and contents\n\
               memory show         Alias for the no-arg form\n\
               memory path         Print just the resolved path\n\
               memory clear        Replace the file with an empty marker\n\
               memory add <note>   Append a timestamped note\n\
               memory help         Show this help\n\n\
             Quick capture: Type `# note` in the composer to append a\n\
             timestamped bullet without firing a turn.",
            path.display()
        )),
    }
}
