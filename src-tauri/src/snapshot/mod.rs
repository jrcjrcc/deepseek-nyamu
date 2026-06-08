//! 工作区快照系统 —— 基于 git 的自动版本管理
//!
//! 在每轮工具驱动的 Agent 对话前后，自动对工作区创建 git 快照，
//! 存储到附属的 git 仓库（位于用户自己的 .git 之外），
//! 使用户可以撤销不需要的更改。
//!
//! 架构：
//! - SnapshotRepo：包装 ~/.deepwhale/snapshots/<workspace_hash>/ 下的 git 仓库
//! - SnapshotId：git commit SHA 的 newtype 包装
//! - pre_turn_snapshot / post_turn_snapshot：引擎的集成点
//! - revert_turn 工具撤销最近一轮更改
//! - /restore 命令恢复到任意指定快照
//!
//! Ported from CodeWhale crates/tui/src/snapshot/
//!
//! Workspace side-git snapshot system.
//!
//! Before and after each tool-backed agent turn, the engine snapshots the
//! working tree into a side-car git repository (a plain git repo outside the
//! user's own `.git`) so the user can revert unwanted changes.

mod paths;
pub mod prune;
pub mod repo;

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;

pub use repo::SnapshotRepo;

/// Opaque snapshot identifier — a git commit SHA hex string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SnapshotId {
    pub sha: String,
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.sha[..std::cmp::min(12, self.sha.len())])
    }
}

impl SnapshotId {
    /// Create from a full SHA hex string.
    #[must_use]
    pub fn new(sha: String) -> Self {
        Self { sha }
    }

    /// Short form (first 12 characters).
    #[must_use]
    pub fn short(&self) -> &str {
        let end = std::cmp::min(12, self.sha.len());
        &self.sha[..end]
    }
}

/// A snapshot has a type label and the commit id.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub label: String,
    pub timestamp: i64,
}

/// The snapshot manager.
pub struct SnapshotManager {
    enabled: bool,
    repo: Arc<Mutex<Option<SnapshotRepo>>>,
    snapshot_count: Arc<std::sync::atomic::AtomicU64>,
}

impl SnapshotManager {
    /// Create a new manager. If `enabled` is false, all operations are no-ops.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            repo: Arc::new(Mutex::new(None)),
            snapshot_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Ensure the underlying repo is initialized for `workspace`.
    async fn ensure_repo(&self, workspace: &Path) -> Result<SnapshotRepo> {
        let mut guard = self.repo.lock().await;
        if let Some(ref repo) = *guard {
            return Ok(repo.clone());
        }
        let repo = SnapshotRepo::open_or_init(workspace)
            .context("failed to init snapshot repo")?;
        let cloned = repo.clone();
        *guard = Some(repo);
        Ok(cloned)
    }

    /// Take a snapshot before a turn begins.
    /// Returns the snapshot id, or None if disabled.
    pub async fn pre_turn_snapshot(&self, workspace: &Path, turn_index: u64) -> Option<SnapshotId> {
        if !self.enabled { return None; }
        let repo = match self.ensure_repo(workspace).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "snapshot: pre-turn init failed");
                return None;
            }
        };
        let msg = format!("pre-turn:{turn_index}");
        match repo.create_snapshot(&msg).await {
            Ok(id) => {
                self.snapshot_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                tracing::debug!(id = %id, turn = turn_index, "snapshot: pre-turn");
                Some(id)
            }
            Err(e) => {
                tracing::warn!(error = %e, "snapshot: pre-turn commit failed");
                None
            }
        }
    }

    /// Take a snapshot after a turn completes.
    pub async fn post_turn_snapshot(&self, workspace: &Path, turn_index: u64) -> Option<SnapshotId> {
        if !self.enabled { return None; }
        let repo = match self.ensure_repo(workspace).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "snapshot: post-turn init failed");
                return None;
            }
        };
        let msg = format!("post-turn:{turn_index}");
        match repo.create_snapshot(&msg).await {
            Ok(id) => {
                self.snapshot_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                tracing::debug!(id = %id, turn = turn_index, "snapshot: post-turn");
                Some(id)
            }
            Err(e) => {
                tracing::warn!(error = %e, "snapshot: post-turn commit failed");
                None
            }
        }
    }

    /// Restore the workspace to a given snapshot.
    pub async fn restore(&self, workspace: &Path, id: &SnapshotId) -> Result<()> {
        let repo = self.ensure_repo(workspace).await?;
        repo.restore(id).await
    }

    /// List all snapshots for this workspace.
    pub async fn list_snapshots(&self, workspace: &Path) -> Vec<Snapshot> {
        let repo = match self.ensure_repo(workspace).await {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        repo.list_snapshots()
    }

    /// Remove snapshots older than `keep_count`.
    pub async fn prune(&self, workspace: &Path, keep_count: usize) -> Result<usize> {
        let repo = self.ensure_repo(workspace).await?;
        repo.prune(keep_count).await
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool { self.enabled }

    /// Number of snapshots taken so far.
    #[must_use]
    pub fn snapshot_count(&self) -> u64 {
        self.snapshot_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}
