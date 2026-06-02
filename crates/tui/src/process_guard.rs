//! Platform-specific child process lifecycle guarantees.
//!
//! Ensures spawned MCP child processes are terminated when the main
//! process exits, even on crash, abort, or SIGKILL (OS permitting).
//!
//! ## Platform strategies
//!
//! | Platform | Primary mechanism | Covers |
//! |----------|-------------------|--------|
//! | **Windows** | Job Object with `KILL_ON_JOB_CLOSE` | All exit paths |
//! | **Linux** | `PR_SET_PDEATHSIG` via `prctl` | All exit paths |
//! | **macOS / other** | atexit fallback | Normal exit / Ctrl+C / SIGTERM |
//!
//! ## Usage
//!
//! 1. Call [`init()`] once at app startup.
//! 2. Call [`register_child(pid)`] after spawning each child.
//! 3. Call [`unregister_child(pid)`] on clean shutdown.
//! 4. On Linux, add [`pre_exec_setup()`] to the `Command` before `.spawn()`.

// All code in this module is only reachable when MCP is configured.
// The compiler can't see that at compile time, so we allow dead-code
// warnings across the whole module.
#![allow(dead_code)]

use std::sync::Mutex;

/// Registered child PIDs for the atexit/signal fallback.
static CHILD_PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

// ───── Public API ─────

/// Initialize process guards. Safe to call multiple times;
/// only the first call takes effect.
pub fn init() {
    static INITIALIZED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if INITIALIZED.set(()).is_err() {
        return;
    }

    // Platform-specific kernel-level mechanism.
    #[cfg(windows)]
    windows_job::init();

    // Register atexit handler for the PID-table fallback.
    // This covers platforms without kernel-level guarantees
    // (macOS) and cases where the kernel mechanism fails.
    register_atexit();
}

/// Register a child PID for guaranteed cleanup on process exit.
///
/// * **Windows**: assigns the child to the global Job Object.
/// * **Linux / macOS / other**: adds to the atexit fallback table.
pub fn register_child(pid: u32) {
    #[cfg(windows)]
    windows_job::register_child(pid);

    if let Ok(mut pids) = CHILD_PIDS.lock() {
        pids.push(pid);
    }
}

/// Remove a child PID from the fallback table on clean shutdown.
pub fn unregister_child(pid: u32) {
    if let Ok(mut pids) = CHILD_PIDS.lock() {
        pids.retain(|&p| p != pid);
    }
}

/// Linux-only: returns a `pre_exec` closure that sets `PR_SET_PDEATHSIG`
/// so the child receives SIGTERM when the parent process ends.
///
/// ```ignore
/// cmd.pre_exec(process_guard::pre_exec_setup());
/// ```
#[cfg(target_os = "linux")]
pub fn pre_exec_setup(
) -> impl FnMut() -> Result<(), std::io::Error> + Send + Sync + 'static {
    || unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
        Ok(())
    }
}

// ───── atexit fallback (all platforms) ─────

fn register_atexit() {
    extern "C" fn cleanup() {
        if let Ok(guard) = CHILD_PIDS.lock() {
            kill_all_pids(&guard);
        }
    }
    #[cfg(unix)]
    unsafe {
        libc::atexit(cleanup);
    }
    #[cfg(windows)]
    unsafe {
        // UCRT `atexit` — always available on Windows via the C runtime.
        unsafe extern "C" {
            fn atexit(func: extern "C" fn()) -> i32;
        }
        atexit(cleanup);
    }
}

fn kill_all_pids(pids: &[u32]) {
    for &pid in pids {
        kill_pid(pid);
    }
}

#[cfg(unix)]
fn kill_pid(pid: u32) {
    unsafe { libc::kill(pid as i32, libc::SIGTERM); }
}

#[cfg(windows)]
fn kill_pid(pid: u32) {
    unsafe {
        unsafe extern "system" {
            fn OpenProcess(
                dw_desired_access: u32,
                b_inherit_handle: i32,
                dw_process_id: u32,
            ) -> *mut std::ffi::c_void;
            fn TerminateProcess(
                h_process: *mut std::ffi::c_void,
                u_exit_code: u32,
            ) -> i32;
            fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
        }
        const PROCESS_TERMINATE: u32 = 0x0001;
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
        }
    }
}

// ───── Windows: Job Object ─────

#[cfg(windows)]
mod windows_job {
    use std::sync::OnceLock;

    type BOOL = i32;
    type DWORD = u32;

    /// Wrapper around Win32 `HANDLE` that implements `Send` and `Sync`.
    /// Required because `OnceLock` needs its type to be `Sync`.
    #[derive(Clone, Copy)]
    struct JobHandle(isize);

    unsafe impl Send for JobHandle {}
    unsafe impl Sync for JobHandle {}

    static JOB: OnceLock<JobHandle> = OnceLock::new();

    /// Create a Job Object with `KILL_ON_JOB_CLOSE`.
    /// When the main process exits, the OS automatically terminates
    /// all processes assigned to this job.
    pub fn init() {
        if JOB.get().is_some() {
            return;
        }
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION =
                std::mem::zeroed();
            info.basic_limit.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
            );
            if ok == 0 {
                CloseHandle(job);
                return;
            }
            JOB.set(JobHandle(job as isize)).ok();
        }
    }

    /// Assign a child process (by PID) to the global Job Object.
    /// Silently ignores failures (e.g., process already in a job).
    pub fn register_child(pid: u32) {
        let &job_handle = match JOB.get() {
            Some(j) => j,
            None => return,
        };
        if job_handle.0 == 0 {
            return;
        }
        let job = job_handle.0 as *mut std::ffi::c_void;
        unsafe {
            let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
            if process.is_null() {
                return;
            }
            AssignProcessToJobObject(job, process);
            CloseHandle(process);
        }
    }

    // ── Win32 constants ──

    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x2000;
    #[allow(non_upper_case_globals)]
    const JobObjectExtendedLimitInformation: DWORD = 9;
    const PROCESS_ALL_ACCESS: DWORD = 0x001F0FFF;

    // ── Win32 structs ──

    #[repr(C)]
    struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: DWORD,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: DWORD,
        affinity: usize,
        priority_class: DWORD,
        scheduling_class: DWORD,
    }

    /// Full struct required by `SetInformationJobObject` with
    /// `JobObjectExtendedLimitInformation`. We only set
    /// `limit_flags`; remaining fields are zero-initialized.
    #[repr(C)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        basic_limit: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        io_info: [u8; 48],
        basic_accounting: [u8; 32],
        basic_and_io_accounting: [u8; 56],
        extended_limit: [u8; 16],
    }

    // ── Win32 FFI ──

    unsafe extern "system" {
        fn CreateJobObjectW(
            lp_job_attributes: *const std::ffi::c_void,
            lp_name: *const u16,
        ) -> *mut std::ffi::c_void;
        fn SetInformationJobObject(
            h_job: *mut std::ffi::c_void,
            job_object_info_class: DWORD,
            lp_job_object_info: *const std::ffi::c_void,
            cb_job_object_info_length: DWORD,
        ) -> BOOL;
        fn AssignProcessToJobObject(
            h_job: *mut std::ffi::c_void,
            h_process: *mut std::ffi::c_void,
        ) -> BOOL;
        fn OpenProcess(
            dw_desired_access: DWORD,
            b_inherit_handle: BOOL,
            dw_process_id: DWORD,
        ) -> *mut std::ffi::c_void;
        fn CloseHandle(h_object: *mut std::ffi::c_void) -> BOOL;
    }
}
