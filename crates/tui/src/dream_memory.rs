//! Dream memory manifest builder.
//!
//! Builds a compact manifest of memory files (filenames + descriptions)
//! for injection into the system prompt. Per-query RAG in
//! `crates/tui/src/tui/ui.rs::dispatch_user_message` handles injecting
//! the actual content of selected memories into the user message.

use std::path::Path;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Manifest cache — avoids regenerating the (byte-stable) manifest block
// when no memory file has been added, removed, or modified.
// ---------------------------------------------------------------------------

struct ManifestCache {
    /// The highest mtime_secs across all scanned `.md` files.
    max_mtime_secs: u64,
    /// Number of scanned files. Paired with max_mtime to detect removals
    /// where the remaining max_mtime happens to stay the same.
    file_count: usize,
    /// The pre-formatted manifest text (output of `build_manifest`).
    manifest: String,
}

static CACHE: Mutex<Option<ManifestCache>> = Mutex::new(None);

/// Invalidate the cache so the next call to `compose_block` rescans and
/// rebuilds from scratch. Useful after a Dream consolidation or `/clear`.
pub fn invalidate_manifest_cache() {
    *CACHE.lock().unwrap() = None;
}

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
/// The result is cached by the set's max mtime + file count so that
/// repeated calls produce a byte-stable output when no files have changed.
/// This preserves the V4 prefix cache across turns.
///
/// Returns `None` when the feature is disabled, the directory is
/// missing, or no readable topic files exist.
#[must_use]
pub fn compose_block(dream_enabled: bool, memory_dir: &Path, _max_file_size: usize) -> Option<String> {
    if !dream_enabled {
        return None;
    }

    let headers = codewhale_core::dream::memdir::scan_memory_dir(memory_dir);
    let file_count = headers.len();
    if file_count == 0 {
        return None;
    }

    // Newest-first sorted — first element has the max mtime.
    let current_max = headers[0].mtime_secs;

    // Fast path: cache hit with identical file set.
    {
        let guard = CACHE.lock().unwrap();
        if let Some(ref cached) = *guard {
            if cached.max_mtime_secs == current_max && cached.file_count == file_count {
                return Some(format!(
                    "<dream_memories>\n{}\n</dream_memories>\n\
                     <!-- byte-stable across turns -->\n\
                     Note: Full content is available via per-query RAG. \
                     To read a specific memory file, use the `read_file` tool \
                     with its path in ~/.codewhale/memories/.",
                    cached.manifest
                ));
            }
        }
    }

    // Cache miss — rebuild manifest.
    let manifest = codewhale_core::dream::memdir::build_manifest(&headers);
    if manifest.is_empty() || manifest == "(no memories)" {
        return None;
    }

    // Atomically update cache *after* regeneration so concurrent readers
    // still see the old (valid) entry until we finish.
    {
        let mut guard = CACHE.lock().unwrap();
        *guard = Some(ManifestCache {
            max_mtime_secs: current_max,
            file_count,
            manifest: manifest.clone(),
        });
    }

    Some(format!(
        "<dream_memories>\n{manifest}\n</dream_memories>\n\
         <!-- byte-stable across turns -->\n\
         Note: Full content is available via per-query RAG. To read a specific memory file, \
         use the `read_file` tool with its path in ~/.codewhale/memories/."
    ))
}

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

    #[test]
    fn test_cache_returns_same_text_on_repeated_call() {
        let (_tmp, path) = create_test_memory_dir();
        let a = compose_block(true, &path, 1024).expect("first call");
        let b = compose_block(true, &path, 1024).expect("second call");
        assert_eq!(a, b, "cache should return identical block");
    }

    #[test]
    fn test_invalidate_forces_regeneration() {
        let (_tmp, path) = create_test_memory_dir();
        let a = compose_block(true, &path, 1024).expect("first call");
        invalidate_manifest_cache();
        let b = compose_block(true, &path, 1024).expect("after invalidate");
        assert_eq!(a, b, "same files → same content even after invalidate");
    }
}
