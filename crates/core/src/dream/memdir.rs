//! Memory directory scanning and relevance selection (RAG).
//!
//! Provides query-time selection of memory files so only the most
//! relevant memories are injected into the prompt, saving tokens and
//! reducing noise. Modeled after Claude Code's `memdir/` subsystem.
//!
//! ## Flow
//!
//! 1. `scan_memory_dir()` — list `.md` files, read frontmatter, return headers
//! 2. `build_manifest()` — format headers as compact text (filename + description)
//! 3. `select_by_query()` — keyword-overlap scoring between query and manifest
//! 4. `compose_blocks()` — read selected files and assemble `<topic>` blocks

use std::fs;
use std::path::{Path, PathBuf};

/// A memory file's header (filename + frontmatter fields).
#[derive(Debug, Clone)]
pub struct MemoryHeader {
    pub filename: String,
    pub file_path: PathBuf,
    pub description: String,
    pub mtime_secs: u64,
}

/// A selected memory ready for prompt injection.
#[derive(Debug, Clone)]
pub struct SelectedMemory {
    pub filename: String,
    pub content: String,
}

/// Maximum number of memory files to scan.
const MAX_SCAN_FILES: usize = 200;

/// Maximum lines to read from each file for frontmatter parsing.
const FRONTMATTER_MAX_LINES: usize = 30;

/// Default number of memories to select per query.
const DEFAULT_SELECTION_LIMIT: usize = 5;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Scan the memory directory and return all discoverable memory headers.
///
/// Reads each `.md` file's first `FRONTMATTER_MAX_LINES` looking for
/// a `description:` field in YAML frontmatter. Files without readable
/// frontmatter are still included with an empty description.
pub fn scan_memory_dir(memory_dir: &Path) -> Vec<MemoryHeader> {
    let dir = match fs::read_dir(memory_dir) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut headers: Vec<MemoryHeader> = Vec::new();

    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .unwrap_or_default();
        // Skip index file — it's metadata, not a topic memory.
        if filename == "MEMORY.md" {
            continue;
        }

        let description = extract_description(&path);
        let mtime_secs = fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        headers.push(MemoryHeader {
            filename,
            file_path: path,
            description,
            mtime_secs,
        });

        if headers.len() >= MAX_SCAN_FILES {
            break;
        }
    }

    // Newest first.
    headers.sort_by(|a, b| b.mtime_secs.cmp(&a.mtime_secs));
    headers
}

/// Build a compact text manifest from memory headers.
///
/// Example output:
/// ```text
/// - traps.md: Known pitfalls and gotchas
/// - architecture.md: Architecture decisions
/// - tips.md: Common tips and tricks
/// ```
pub fn build_manifest(headers: &[MemoryHeader]) -> String {
    if headers.is_empty() {
        return String::from("(no memories)");
    }

    let mut lines: Vec<String> = Vec::with_capacity(headers.len());
    for h in headers {
        if h.description.is_empty() {
            lines.push(format!("- {}", h.filename));
        } else {
            lines.push(format!("- {}: {}", h.filename, h.description));
        }
    }
    lines.join("\n")
}

/// Select up to `limit` memory files relevant to the query.
///
/// Uses simple keyword-overlap scoring (token intersection / union).
/// Files with higher description-overlap to the query rank first.
/// This is a synchronous, zero-API-call selector — adequate for v1;
/// future versions may call a cheap model for semantic selection.
pub fn select_by_query(query: &str, headers: &[MemoryHeader]) -> Vec<MemoryHeader> {
    select_by_query_limit(query, headers, DEFAULT_SELECTION_LIMIT)
}

/// Same as [`select_by_query`] with an explicit limit.
pub fn select_by_query_limit(
    query: &str,
    headers: &[MemoryHeader],
    limit: usize,
) -> Vec<MemoryHeader> {
    if headers.is_empty() || limit == 0 {
        return Vec::new();
    }

    let query_tokens: Vec<String> = tokenize(query);

    // Score each header by token overlap.
    struct Scored {
        idx: usize,
        score: f64,
    }

    let mut scored: Vec<Scored> = headers
        .iter()
        .enumerate()
        .map(|(idx, h)| {
            let haystack = format!("{} {}", h.filename.replace(".md", ""), h.description);
            let haystack_tokens = tokenize(&haystack);
            let score = jaccard_similarity(&query_tokens, &haystack_tokens);
            Scored { idx, score }
        })
        .filter(|s| s.score > 0.0)
        .collect();

    // Sort by score descending.
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let count = limit.min(scored.len());
    scored[..count]
        .iter()
        .map(|s| headers[s.idx].clone())
        .collect()
}

/// Read selected memory files and assemble `<topic>` blocks.
///
/// Each file is truncated to `max_bytes` if it exceeds the limit,
/// with a truncation marker appended.
pub fn compose_blocks(
    selected: &[MemoryHeader],
    max_bytes: usize,
) -> String {
    let mut blocks = String::from("<dream_memories>\n");

    for mem in selected {
        let content = read_truncated(&mem.file_path, max_bytes);
        if content.trim().is_empty() {
            continue;
        }
        blocks.push_str(&format!(
            "<topic name=\"{}\">\n{content}\n</topic>\n",
            mem.filename.replace(".md", "")
        ));
    }

    blocks.push_str("</dream_memories>");
    blocks
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Extract the `description:` field from YAML frontmatter.
///
/// Reads the first `FRONTMATTER_MAX_LINES` lines. Returns empty string
/// if no frontmatter or no description field is found.
fn extract_description(path: &Path) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Must start with `---`.
    let content = content.trim_start();
    if !content.starts_with("---") {
        return String::new();
    }

    let after_opener = &content[3..];
    let end = after_opener.find("\n---").unwrap_or(after_opener.len());
    let frontmatter = &after_opener[..end];

    for line in frontmatter.lines().take(FRONTMATTER_MAX_LINES) {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("description:") {
            return value.trim().trim_matches('"').trim_matches('\'').to_string();
        }
        if let Some(value) = line.strip_prefix("description: ") {
            return value.trim().trim_matches('"').trim_matches('\'').to_string();
        }
    }

    String::new()
}

/// Read a file, truncating to `max_bytes` if it exceeds the limit.
fn read_truncated(path: &Path, max_bytes: usize) -> String {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    if content.len() <= max_bytes {
        return content;
    }

    // Truncate at character boundary.
    let mut cutoff = max_bytes;
    while !content.is_char_boundary(cutoff) {
        cutoff -= 1;
    }
    let omitted = content.len() - cutoff;
    let head = &content[..cutoff];
    let marker = format!("\n\n<!-- truncated: omitted {omitted} bytes from {} -->", path.display());
    format!("{head}{marker}")
}

/// Tokenize a string into lowercase words.
///
/// - CJK characters (U+2E80+) are split individually (each character is
///   a meaningful semantic unit).
/// - Non-CJK text (English, etc.) is split on non-alphanumeric boundaries
///   into word tokens.
/// - Tokens must be at least 2 bytes long (filters out single ASCII chars
///   like "a" or "I", but keeps single CJK chars).
fn tokenize(s: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut buf = String::new();

    for ch in s.chars() {
        let is_cjk = (ch as u32) >= 0x2E80;
        if is_cjk {
            // Flush any accumulated ASCII/non-CJK buffer.
            if buf.len() >= 2 {
                result.push(buf.clone());
            }
            buf.clear();
            // Each CJK character is its own token (if non-whitespace).
            if !ch.is_whitespace() {
                result.push(ch.to_string().to_lowercase());
            }
        } else if ch.is_alphanumeric() {
            buf.push(ch);
        } else {
            // Separator character.
            if buf.len() >= 2 {
                result.push(buf.clone());
            }
            buf.clear();
        }
    }
    if buf.len() >= 2 {
        result.push(buf);
    }

    result
}

/// Jaccard similarity: |intersection| / |union|.
fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let set_a: std::collections::HashSet<&str> = a.iter().map(String::as_str).collect();
    let set_b: std::collections::HashSet<&str> = b.iter().map(String::as_str).collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();

        // traps.md with frontmatter
        fs::write(
            path.join("traps.md"),
            "---\ndescription: Known pitfalls and gotchas\n---\n\n## Traps\n\n- NPE in command_safety.rs\n",
        )
        .unwrap();

        // architecture.md without frontmatter
        fs::write(
            path.join("architecture.md"),
            "## Architecture\n\nCore is agent→tools→state.\n",
        )
        .unwrap();

        // tips.md with frontmatter
        fs::write(
            path.join("tips.md"),
            "---\ndescription: Common tips and tricks\n---\n\n## Tips\n\n1. Use `--yolo` for speed\n",
        )
        .unwrap();

        // MEMORY.md — should be excluded
        fs::write(path.join("MEMORY.md"), "# Index\n\n- traps.md\n").unwrap();

        (dir, path)
    }

    #[test]
    fn test_scan_excludes_memory_index() {
        let (_tmp, path) = create_test_dir();
        let headers = scan_memory_dir(&path);
        assert!(!headers.iter().any(|h| h.filename == "MEMORY.md"));
    }

    #[test]
    fn test_scan_reads_frontmatter() {
        let (_tmp, path) = create_test_dir();
        let headers = scan_memory_dir(&path);

        let traps = headers.iter().find(|h| h.filename == "traps.md");
        assert!(traps.is_some());
        assert_eq!(traps.unwrap().description, "Known pitfalls and gotchas");

        let tips = headers.iter().find(|h| h.filename == "tips.md");
        assert!(tips.is_some());
        assert_eq!(tips.unwrap().description, "Common tips and tricks");
    }

    #[test]
    fn test_scan_file_without_frontmatter_gets_empty_description() {
        let (_tmp, path) = create_test_dir();
        let headers = scan_memory_dir(&path);

        let arch = headers.iter().find(|h| h.filename == "architecture.md");
        assert!(arch.is_some());
        assert!(arch.unwrap().description.is_empty());
    }

    #[test]
    fn test_build_manifest() {
        let headers = vec![
            MemoryHeader {
                filename: "traps.md".into(),
                file_path: PathBuf::new(),
                description: "Pitfalls".into(),
                mtime_secs: 1000,
            },
            MemoryHeader {
                filename: "tips.md".into(),
                file_path: PathBuf::new(),
                description: "Tips".into(),
                mtime_secs: 999,
            },
        ];
        let manifest = build_manifest(&headers);
        assert!(manifest.contains("traps.md: Pitfalls"));
        assert!(manifest.contains("tips.md: Tips"));
    }

    #[test]
    fn test_select_by_query_matches_description() {
        let headers = vec![
            MemoryHeader {
                filename: "traps.md".into(),
                file_path: PathBuf::new(),
                description: "Known pitfalls and gotchas in the codebase".into(),
                mtime_secs: 1000,
            },
            MemoryHeader {
                filename: "tips.md".into(),
                file_path: PathBuf::new(),
                description: "Performance optimization tips".into(),
                mtime_secs: 999,
            },
        ];

        let result = select_by_query("performance tips", &headers);
        assert_eq!(result.len(), 1); // only tips.md has keyword overlap
        assert_eq!(result[0].filename, "tips.md");
    }

    #[test]
    fn test_select_by_query_empty_returns_empty() {
        let headers = vec![MemoryHeader {
            filename: "traps.md".into(),
            file_path: PathBuf::new(),
            description: "Pitfalls".into(),
            mtime_secs: 1000,
        }];
        let result = select_by_query("zzzzz unmatched", &headers);
        assert!(result.is_empty());
    }

    #[test]
    fn test_tokenize_removes_short_words() {
        let tokens = tokenize("a bb ccc dddd");
        assert!(!tokens.contains(&"a".to_string()));
        assert!(tokens.contains(&"bb".to_string())); // kept (len >= 2)
        assert!(tokens.contains(&"ccc".to_string()));
        assert!(tokens.contains(&"dddd".to_string()));
    }

    #[test]
    fn test_compose_blocks() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();

        fs::write(path.join("test.md"), "Hello memory").unwrap();

        let headers = vec![MemoryHeader {
            filename: "test.md".into(),
            file_path: path.join("test.md"),
            description: "Test".into(),
            mtime_secs: 0,
        }];

        let block = compose_blocks(&headers, 1024);
        assert!(block.contains("<topic name=\"test\">"));
        assert!(block.contains("Hello memory"));
        assert!(block.contains("</dream_memories>"));
    }
}
