use std::path::PathBuf;

use serde::Deserialize;

/// Configuration for the Dream memory consolidation subsystem.
/// Corresponds to `[dream]` section in config.toml.
///
/// Default is opt-out — when the `[dream]` table is absent,
/// consolidation is enabled with defaults.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DreamConfig {
    /// Master switch. Default `true`.
    pub enabled: bool,

    /// Directory for memory files. `~` is expanded to the user's home
    /// directory at runtime.
    /// Default: `~/.codewhale/memories`.
    pub memory_dir: PathBuf,

    /// Maximum size of any single memory file, in bytes.
    /// Files larger than this are truncated during the Prune phase.
    /// Default: 16 384 (16 KiB).
    pub max_memory_file_size: usize,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_dir: PathBuf::from("~/.codewhale/memories"),
            max_memory_file_size: 16 * 1024,
        }
    }
}

impl DreamConfig {
    /// Resolve `~` in `memory_dir` and return the canonical path.
    pub fn resolved_memory_dir(&self) -> PathBuf {
        expand_tilde(&self.memory_dir)
    }
}

fn expand_tilde(path: &PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let replaced = s.replacen('~', &home.to_string_lossy(), 1);
            return PathBuf::from(replaced);
        }
    }
    path.clone()
}
