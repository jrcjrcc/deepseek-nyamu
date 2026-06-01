//! File-based consolidation lock.
//!
//! Uses atomic [`File::create_new`] so two processes cannot race.
//! The lock file stores the PID and a Unix‑epoch timestamp for
//! diagnostic purposes. Staleness is determined purely by mtime:
//! if the file is older than [`LOCK_TIMEOUT`] it is removed and
//! re‑acquired.
//!
//! The lock file is cleaned up on [`Drop`]. A panic / hard‑kill
//! leaves a stale file that will be reclaimed after the timeout.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};

const LOCK_FILE: &str = ".consolidate-lock";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Outcome of a [`ConsolidationLock::try_acquire`] call.
#[derive(Debug)]
pub enum LockStatus {
    /// Lock acquired successfully. The caller should proceed with
    /// consolidation and hold the returned value until finished.
    Acquired(ConsolidationLock),

    /// Another process (or this process earlier) holds the lock
    /// within the timeout window.
    Held,
}

/// A file‑based, process‑scope lock for memory consolidation.
///
/// While this value is alive the lock file exists. Dropping it
/// deletes the file.
#[derive(Debug)]
pub struct ConsolidationLock {
    path: PathBuf,
}

impl ConsolidationLock {
    /// Try to acquire the consolidation lock for the given memory
    /// directory.
    ///
    /// 1. Atomically create `.consolidate-lock` inside `memory_dir`.
    /// 2. On `AlreadyExists` — check mtime.
    ///    a. mtime older than `timeout` → stale, remove + retry.
    ///    b. mtime within `timeout` → return [`LockStatus::Held`].
    /// 3. On success — write PID + timestamp and return the lock.
    pub fn try_acquire(memory_dir: &Path, timeout: Duration) -> Result<LockStatus> {
        let lock_path = memory_dir.join(LOCK_FILE);

        // Ensure the parent directory exists
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match fs::File::create_new(&lock_path) {
            Ok(mut file) => {
                let pid = std::process::id();
                let now_secs = unix_now_secs();
                let content = format!("{pid}\n{now_secs}\n");
                file.write_all(content.as_bytes())
                    .context("write lock file")?;
                drop(file);
                Ok(LockStatus::Acquired(ConsolidationLock { path: lock_path }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if is_lock_stale(&lock_path, timeout) {
                    let _ = fs::remove_file(&lock_path);
                    return Self::try_acquire(memory_dir, timeout);
                }
                Ok(LockStatus::Held)
            }
            Err(e) => Err(e).context("create consolidation lock file")?,
        }
    }

    /// Return the mtime of the last lock acquisition, if a lock file
    /// exists and is readable.
    pub fn last_consolidated_at(memory_dir: &Path) -> Option<SystemTime> {
        let lock_path = memory_dir.join(LOCK_FILE);
        fs::metadata(&lock_path).ok()?.modified().ok()
    }

    /// Return the PID stored in the lock file, if any.
    pub fn read_pid(memory_dir: &Path) -> Option<u32> {
        let lock_path = memory_dir.join(LOCK_FILE);
        let mut buf = String::new();
        fs::File::open(&lock_path).ok()?.read_to_string(&mut buf).ok()?;
        buf.lines().next()?.parse().ok()
    }
}

impl Drop for ConsolidationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_lock_stale(lock_path: &Path, timeout: Duration) -> bool {
    let meta = match fs::metadata(lock_path) {
        Ok(m) => m,
        Err(_) => return true, // can't read → treat as stale
    };
    let modified = match meta.modified() {
        Ok(t) => t,
        Err(_) => return true,
    };
    match SystemTime::now().duration_since(modified) {
        Ok(elapsed) => elapsed >= timeout,
        Err(_) => true, // clock went backwards → stale
    }
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_acquire_and_release() {
        let dir = tempfile::tempdir().unwrap();

        // First acquire succeeds — hold the lock
        let lock = match ConsolidationLock::try_acquire(dir.path(), Duration::from_secs(3600))
            .unwrap()
        {
            LockStatus::Acquired(l) => l,
            LockStatus::Held => panic!("expected Acquired"),
        };

        // Lock file exists while we hold it
        assert!(dir.path().join(LOCK_FILE).exists());

        // Drop → file removed
        drop(lock);
        assert!(!dir.path().join(LOCK_FILE).exists());
    }

    #[test]
    fn test_held_when_lock_active() {
        let dir = tempfile::tempdir().unwrap();

        let _lock = match ConsolidationLock::try_acquire(dir.path(), Duration::from_secs(3600))
            .unwrap()
        {
            LockStatus::Acquired(l) => l,
            LockStatus::Held => panic!("expected Acquired"),
        };

        // Second attempt should be Held
        match ConsolidationLock::try_acquire(dir.path(), Duration::from_secs(3600)).unwrap() {
            LockStatus::Acquired(_) => panic!("expected Held"),
            LockStatus::Held => {} // correct
        }
    }

    #[test]
    fn test_stale_lock_reclaimed() {
        let dir = tempfile::tempdir().unwrap();

        // Create a lock file with old mtime
        let lock_path = dir.path().join(LOCK_FILE);
        fs::write(&lock_path, "0\n0\n").unwrap();

        // Set mtime far in the past
        let old_time = SystemTime::now() - Duration::from_secs(7200);
        let filetime = filetime::FileTime::from_system_time(old_time);
        filetime::set_file_mtime(&lock_path, filetime).unwrap();

        // Should reclaim (stale because mtime > 1h ago)
        match ConsolidationLock::try_acquire(dir.path(), Duration::from_secs(3600)).unwrap() {
            LockStatus::Acquired(_) => {} // stale was reclaimed
            LockStatus::Held => panic!("expected stale lock to be reclaimed"),
        }
    }
}
