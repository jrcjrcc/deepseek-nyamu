//! 附属 git 仓库 —— 工作区快照的存储后端
//!
//! 在用户工作区之外维护一个纯 git 仓库（非 bare repo），
//! 每次快照通过 git commit 保存在附属仓库中。
//!
//! 仓库位置：~/.deepwhale/snapshots/<workspace_hash>/
//! 提交者信息固定为 deepwhale-snapshots。
//! git 操作通过 Mutex 序列化，避免并发冲突。
//!
//! Ported from CodeWhale crates/tui/src/snapshot/repo.rs
//!
//! Side-git repository wrapper for workspace snapshots.
//!
//! Manages a plain git repository (not a bare repo) that lives outside the
//! user's own `.git`. Snapshots are git commits in this side-car repo.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Result, bail};
use tokio::sync::Mutex;

use super::paths::SnapshotPaths;
use super::{Snapshot, SnapshotId};

const SNAPSHOT_USER_NAME: &str = "deepwhale-snapshots";
const SNAPSHOT_USER_EMAIL: &str = "snapshots@deepwhale.local";

/// Wrapper around the per-workspace side-git repo.
#[derive(Clone)]
pub struct SnapshotRepo {
    git_dir: PathBuf,
    work_tree: PathBuf,
    // Serialize git operations so we never have concurrent commits
    lock: Arc<Mutex<()>>,
}

impl SnapshotRepo {
    /// Open an existing snapshot repo or initialize a new one.
    pub fn open_or_init(workspace: &Path) -> std::io::Result<Self> {
        let paths = SnapshotPaths::for_workspace(workspace);
        std::fs::create_dir_all(&paths.root)?;

        if !paths.git_dir.exists() {
            run_git(&paths.git_dir, &paths.work_tree, &["init", "--quiet"])?;
            run_git(&paths.git_dir, &paths.work_tree, &["config", "user.name", SNAPSHOT_USER_NAME])?;
            run_git(&paths.git_dir, &paths.work_tree, &["config", "user.email", SNAPSHOT_USER_EMAIL])?;
            run_git(&paths.git_dir, &paths.work_tree, &["config", "gc.auto", "0"])?;
        }

        Ok(Self {
            git_dir: paths.git_dir,
            work_tree: paths.work_tree,
            lock: Arc::new(Mutex::new(())),
        })
    }

    /// Create a snapshot commit of the user's workspace.
    pub async fn create_snapshot(&self, label: &str) -> Result<SnapshotId> {
        let _guard = self.lock.lock().await;

        // Copy workspace files into the snapshot repo tree
        let workspace = self.discover_workspace()?;

        // Copy workspace to snapshot work_tree (excluding .git dirs)
        self.copy_workspace_to_tree(&workspace)?;

        // Stage everything
        run_git(&self.git_dir, &self.work_tree, &["add", "--all"])?;

        // Check if anything changed
        let status = run_git_capture(&self.git_dir, &self.work_tree, &["status", "--porcelain"])?;
        if status.trim().is_empty() {
            bail!("nothing to snapshot");
        }

        // Commit
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let msg = format!("{label}\n\n{timestamp}");
        let _output = run_git_capture(
            &self.git_dir,
            &self.work_tree,
            &["commit", "--allow-empty", "-m", &msg],
        )?;

        // Extract SHA
        let sha = run_git_capture(
            &self.git_dir,
            &self.work_tree,
            &["rev-parse", "HEAD"],
        )?
        .trim()
        .to_string();

        Ok(SnapshotId::new(sha))
    }

    /// Restore the user's workspace to the state captured in a snapshot.
    pub async fn restore(&self, id: &SnapshotId) -> Result<()> {
        let _guard = self.lock.lock().await;
        let workspace = self.discover_workspace()?;

        run_git(&self.git_dir, &self.work_tree, &["checkout", &id.sha, "--", ":/"])?;

        // Copy files from snapshot work tree back to workspace
        self.copy_tree_to_workspace(&workspace)?;

        // Reset the snapshot tree back to HEAD
        run_git(&self.git_dir, &self.work_tree, &["reset", "--hard", "HEAD"])?;

        Ok(())
    }

    /// List all snapshots in reverse chronological order.
    pub fn list_snapshots(&self) -> Vec<Snapshot> {
        let output = run_git_capture(
            &self.git_dir,
            &self.work_tree,
            &["log", "--format=%H %ct %s", "--all", "--reverse"],
        )
        .unwrap_or_default();

        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() < 3 { return None; }
                let sha = parts[0].to_string();
                let timestamp = parts[1].parse::<i64>().unwrap_or(0);
                let label = parts[2].to_string();
                Some(Snapshot { id: SnapshotId::new(sha), label, timestamp })
            })
            .collect()
    }

    /// Prune old snapshots, keeping the most recent `keep_count`.
    pub async fn prune(&self, keep_count: usize) -> Result<usize> {
        let _guard = self.lock.lock().await;
        let snapshots = self.list_snapshots();
        if snapshots.len() <= keep_count {
            return Ok(0);
        }

        let to_remove = snapshots.len() - keep_count;
        for snap in &snapshots[..to_remove] {
            let _ = run_git(&self.git_dir, &self.work_tree, &[
                "update-ref",
                "-d",
                &format!("refs/heads/snapshot-{}", snap.id.short()),
            ]);
        }
        Ok(to_remove)
    }

    fn discover_workspace(&self) -> Result<PathBuf> {
        // Infer workspace from the snapshot dir path structure
        let current = self.work_tree.parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Ok(current)
    }

    fn copy_workspace_to_tree(&self, workspace: &Path) -> std::io::Result<()> {
        copy_dir(workspace, &self.work_tree, true)?;
        Ok(())
    }

    fn copy_tree_to_workspace(&self, workspace: &Path) -> std::io::Result<()> {
        copy_dir(&self.work_tree, workspace, true)?;
        Ok(())
    }
}

fn run_git(git_dir: &Path, work_tree: &Path, args: &[&str]) -> std::io::Result<()> {
    let output = std::process::Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .arg(format!("--work-tree={}", work_tree.display()))
        .args(args)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("git {} failed: {stderr}", args.join(" ")),
        ));
    }
    Ok(())
}

fn run_git_capture(git_dir: &Path, work_tree: &Path, args: &[&str]) -> std::io::Result<String> {
    let output = std::process::Command::new("git")
        .arg(format!("--git-dir={}", git_dir.display()))
        .arg(format!("--work-tree={}", work_tree.display()))
        .args(args)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("git {} failed: {stderr}", args.join(" ")),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Recursive directory copy without `walkdir` crate.
fn copy_dir(src: &Path, dst: &Path, exclude_git: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;

    // Use a simple stack-based traversal instead of walkdir
    let mut stack = vec![src.to_path_buf()];
    while let Some(current_src) = stack.pop() {
        let relative = current_src.strip_prefix(src).unwrap_or(&current_src);
        let current_dst = dst.join(relative);

        if current_src.is_dir() {
            std::fs::create_dir_all(&current_dst)?;
            for entry in std::fs::read_dir(&current_src)? {
                let entry = entry?;
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if exclude_git && (name_str == ".git" || name_str == ".snapshots" || name_str.starts_with(".git_")) {
                    continue;
                }
                stack.push(entry.path());
            }
        } else {
            std::fs::copy(&current_src, &current_dst)?;
        }
    }
    Ok(())
}
