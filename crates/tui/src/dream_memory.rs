//! Dream memory manifest builder.
//!
//! Builds a compact manifest of memory files (filenames + descriptions)
//! for injection into the system prompt. Per-query RAG in
//! `crates/tui/src/tui/ui.rs::dispatch_user_message` handles injecting
//! the actual content of selected memories into the user message.

use std::path::Path;

use chrono::Utc;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load and assemble Dream memories into a `<dream_memories>` block.
///
/// Now returns a compact **manifest** (filenames + descriptions) instead of
/// full file content, saving ~14KB per turn. Per-query RAG in
/// [`dispatch_user_message`] handles injecting the actual content of
/// selected memories into the user message.
///
/// Returns `None` when the feature is disabled, the directory is
/// missing, or no readable topic files exist.
#[must_use]
pub fn compose_block(dream_enabled: bool, memory_dir: &Path, _max_file_size: usize) -> Option<String> {
    if !dream_enabled {
        return None;
    }

    let headers = codewhale_core::dream::memdir::scan_memory_dir(memory_dir);
    if headers.is_empty() {
        return None;
    }

    let manifest = codewhale_core::dream::memdir::build_manifest(&headers);
    if manifest.is_empty() || manifest == "(no memories)" {
        return None;
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    Some(format!(
        "<dream_memories>\n{manifest}\n</dream_memories>\n<!-- loaded at {now} -->\n\
         Note: Full content is available via per-query RAG. To read a specific memory file, \
         use the `read_file` tool with its path in ~/.codewhale/memories/."
    ))
}

// (no internal helpers — scanning is delegated to `codewhale_core::dream::memdir`)

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_memory_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let mem_path = dir.path().to_path_buf();
        // Files need frontmatter for description extraction
        fs::write(
            mem_path.join("traps.md"),
            "---\ndescription: Known pitfalls and gotchas\n---\n\n## Traps\n\n- NPE in command_safety.rs\n",
        )
        .unwrap();
        fs::write(
            mem_path.join("architecture.md"),
            "---\ndescription: Architecture decisions\n---\n\n## Architecture\n\nCore is agent→tools→state.\n",
        )
        .unwrap();
        fs::write(mem_path.join("MEMORY.md"), "# Index\n\n- traps.md\n").unwrap();
        (dir, mem_path)
    }

    #[test]
    fn test_compose_block_disabled() {
        let (_tmp, path) = create_test_memory_dir();
        assert!(compose_block(false, &path, 1024).is_none());
    }

    #[test]
    fn test_compose_block_loads_manifest() {
        let (_tmp, path) = create_test_memory_dir();
        let block = compose_block(true, &path, 1024);
        assert!(block.is_some());
        let text = block.unwrap();
        assert!(text.contains("<dream_memories>"));
        assert!(text.contains("traps"));
        assert!(text.contains("architecture"));
        assert!(text.contains("</dream_memories>"));
        // Manifest only — no full content
        assert!(!text.contains("NPE in command_safety.rs"));
    }

    #[test]
    fn test_compose_block_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(compose_block(true, dir.path(), 1024).is_none());
    }

    #[test]
    fn test_compose_block_manifest_without_frontmatter_includes_filename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path();
        // File without frontmatter
        fs::write(path.join("notes.md"), "some notes here").unwrap();
        let block = compose_block(true, path, 1024);
        assert!(block.is_some());
        let text = block.unwrap();
        assert!(text.contains("notes"));
    }
}
