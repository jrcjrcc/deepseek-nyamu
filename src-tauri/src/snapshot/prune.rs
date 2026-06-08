//! Pruning strategy for old snapshots.
//! Ported from CodeWhale crates/tui/src/snapshot/prune.rs

/// Default maximum number of snapshots to keep per workspace.
pub const DEFAULT_KEEP_COUNT: usize = 100;

/// Pruning result.
pub struct PruneResult {
    pub removed: usize,
    pub remaining: usize,
}
