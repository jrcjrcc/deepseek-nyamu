//! Per-workspace snapshot repository paths.
//! Ported from CodeWhale crates/tui/src/snapshot/paths.rs
//! No external dependencies (dirs, hex) — uses std only.

use std::path::{Path, PathBuf};

/// Root directory for all snapshot repos.
fn snapshots_root() -> PathBuf {
    // Try common data dirs without the `dirs` crate
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local").join("share").join("deepwhale").join("snapshots");
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile).join("AppData").join("Roaming").join("deepwhale").join("snapshots");
    }
    if let Ok(home) = std::env::var("HOMEDRIVE").and_then(|d| std::env::var("HOMEPATH").map(move |p| format!("{d}{p}"))) {
        return PathBuf::from(home).join(".deepwhale").join("snapshots");
    }
    PathBuf::from(".deepwhale").join("snapshots")
}

/// Hash the workspace path to a safe directory name (std-only SHA256 hex).
fn workspace_hash(workspace: &Path) -> String {
    use sha2::{Sha256, Digest};
    let abs = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(workspace)
    };
    let canonical = abs.canonicalize().unwrap_or(abs);
    let mut hasher = Sha256::new();
    hasher.update(canonical.display().to_string().as_bytes());
    let result = hasher.finalize();
    // hex encode the first 16 bytes manually (no `hex` crate)
    let bytes = &result[..16];
    let hex_chars: String = bytes.iter()
        .flat_map(|b| {
            let hi = b >> 4;
            let lo = b & 0x0f;
            [hex_nibble(hi), hex_nibble(lo)]
        })
        .collect();
    hex_chars // 32 hex chars
}

fn hex_nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => '0',
    }
}

/// Paths for a single workspace's snapshot repo.
pub struct SnapshotPaths {
    /// The snapshot repo root: `~/.deepwhale/snapshots/<hash>/`
    pub root: PathBuf,
    /// Git directory: `root/.git`
    pub git_dir: PathBuf,
    /// Working tree (same as root for side-git repos)
    pub work_tree: PathBuf,
}

impl SnapshotPaths {
    #[must_use]
    pub fn for_workspace(workspace: &Path) -> Self {
        let root = snapshots_root().join(workspace_hash(workspace));
        Self {
            git_dir: root.join(".git"),
            work_tree: root.clone(),
            root,
        }
    }
}
