use std::path::PathBuf;

use codewhale_state::{SessionSource, StateStore, ThreadListFilters, ThreadMetadata, ThreadStatus};

fn temp_state_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "deepseek_state_test_{}_{}_{}.db",
        label,
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ))
}

#[test]
fn upsert_and_resume_thread_metadata() {
    let path = temp_state_path("upsert_resume");
    let store = StateStore::open(Some(path.clone())).expect("open state store");
    let now = chrono::Utc::now().timestamp();
    let thread = ThreadMetadata {
        id: "thread-test-1".to_string(),
        rollout_path: Some(PathBuf::from("/tmp/rollout.jsonl")),
        preview: "hello".to_string(),
        ephemeral: false,
        model_provider: "deepseek".to_string(),
        created_at: now,
        updated_at: now,
        status: ThreadStatus::Running,
        path: Some(PathBuf::from("/tmp/project")),
        cwd: PathBuf::from("/tmp/project"),
        cli_version: "0.0.0-test".to_string(),
        source: SessionSource::Interactive,
        name: Some("Test Thread".to_string()),
        sandbox_policy: Some("workspace-write".to_string()),
        approval_mode: Some("on-request".to_string()),
        archived: false,
        archived_at: None,
        git_sha: None,
        git_branch: None,
        git_origin_url: None,
        memory_mode: Some("extended".to_string()),
    };
    store.upsert_thread(&thread).expect("upsert thread");

    let loaded = store
        .get_thread("thread-test-1")
        .expect("read thread")
        .expect("thread must exist");
    assert_eq!(loaded.id, "thread-test-1");
    assert_eq!(loaded.name.as_deref(), Some("Test Thread"));
    assert_eq!(loaded.memory_mode.as_deref(), Some("extended"));
    assert_eq!(
        loaded.rollout_path,
        Some(PathBuf::from("/tmp/rollout.jsonl"))
    );

    store
        .mark_archived("thread-test-1")
        .expect("archive thread");
    let archived = store
        .get_thread("thread-test-1")
        .expect("read archived thread")
        .expect("thread exists after archive");
    assert!(archived.archived);

    let listed = store
        .list_threads(ThreadListFilters {
            include_archived: true,
            limit: Some(10),
        })
        .expect("list threads");
    assert!(!listed.is_empty());
}

#[test]
fn pragma_verify() {
    // Verify that init_schema() applies the expected PRAGMAs.
    // WAL mode persists to the DB file; per-connection PRAGMAs are
    // set on every open and can be checked on a fresh connection.
    let path = temp_state_path("pragma_verify");
    let store = StateStore::open(Some(path.clone())).expect("open state store");

    // Open a separate connnection to verify PRAGMAs without using StateStore.
    let conn = rusqlite::Connection::open(&path).expect("open raw connection");

    let journal_mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .expect("read journal_mode");
    assert_eq!(journal_mode.to_uppercase(), "WAL", "WAL mode must be enabled");

    let busy_timeout: i32 = conn
        .pragma_query_value(None, "busy_timeout", |row| row.get(0))
        .expect("read busy_timeout");
    assert_eq!(busy_timeout, 5000, "busy_timeout must be 5000ms");

    let foreign_keys: bool = conn
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("read foreign_keys");
    assert!(foreign_keys, "foreign_keys must be ON");

    drop(conn);

    // Verify that StateStore can still operate normally.
    let thread = ThreadMetadata {
        id: "pragma-test-thread".to_string(),
        rollout_path: None,
        preview: "pragma test".to_string(),
        ephemeral: false,
        model_provider: "deepseek".to_string(),
        created_at: 0,
        updated_at: 0,
        status: ThreadStatus::Running,
        path: None,
        cwd: std::env::temp_dir(),
        cli_version: "0.0.0-test".to_string(),
        source: SessionSource::Interactive,
        name: None,
        sandbox_policy: None,
        approval_mode: None,
        archived: false,
        archived_at: None,
        git_sha: None,
        git_branch: None,
        git_origin_url: None,
        memory_mode: None,
    };
    store.upsert_thread(&thread).expect("upsert after pragma verify");
    let loaded = store
        .get_thread("pragma-test-thread")
        .expect("read after pragma verify")
        .expect("thread must exist");
    assert_eq!(loaded.id, "pragma-test-thread");

    std::fs::remove_file(&path).ok();
}

#[test]
fn concurrent_upsert() {
    // Multiple threads concurrently upserting into the same DB file.
    // WAL mode + busy_timeout should prevent SQLITE_BUSY crashes.
    // The DB is initialized single-threaded first, then shared.
    let path = temp_state_path("concurrent_upsert");

    // Single-threaded init: create schema and enable WAL mode.
    let _init = StateStore::open(Some(path.clone())).expect("init state store");

    let num_threads = 8;
    let num_writes = 25;

    std::thread::scope(|scope| {
        for t in 0..num_threads {
            let p = path.clone();
            scope.spawn(move || {
                let store = StateStore::open(Some(p)).expect("open state store");
                for i in 0..num_writes {
                    let thread = ThreadMetadata {
                        id: format!("concurrent-{}-{}", t, i),
                        rollout_path: None,
                        preview: format!("thread {}/{}", t, i),
                        ephemeral: false,
                        model_provider: "deepseek".to_string(),
                        created_at: i as i64,
                        updated_at: i as i64,
                        status: ThreadStatus::Running,
                        path: None,
                        cwd: std::env::temp_dir(),
                        cli_version: "0.0.0-test".to_string(),
                        source: SessionSource::Interactive,
                        name: Some(format!("Thread {}-{}", t, i)),
                        sandbox_policy: None,
                        approval_mode: None,
                        archived: false,
                        archived_at: None,
                        git_sha: None,
                        git_branch: None,
                        git_origin_url: None,
                        memory_mode: None,
                    };
                    store.upsert_thread(&thread).expect(&format!("upsert {}/{}", t, i));
                }
            });
        }
    });

    // Verify total count
    let store = StateStore::open(Some(path.clone())).expect("open for verify");
    let all = store
        .list_threads(ThreadListFilters {
            include_archived: true,
            limit: Some((num_threads * num_writes + 1) as usize),
        })
        .expect("list all threads");
    assert_eq!(
        all.len() as u32,
        num_threads * num_writes,
        "all concurrent writes should persist"
    );

    std::fs::remove_file(&path).ok();
}
