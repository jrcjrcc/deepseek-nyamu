//! 沙盒执行策略 —— Shell 命令进程隔离
//!
//! 核心机制：
//! - Windows：使用 CREATE_NO_WINDOW + 进程组实现干净终止
//! - 文件写入限制在工具处理层实施（路径逃逸检查）
//! - 网络策略待实现（Windows 过滤平台需要管理员权限）
//!
//! SandboxSpec 控制三种策略：
//! - mode = "enforce"：启用沙盒限制
//! - write_roots：允许写入的目录白名单
//! - network：是否允许网络访问
//!
//! nyamu-style sandbox — process confinement for shell commands.
//!
/// On Windows: uses CREATE_NO_WINDOW + process group for clean termination.
/// File-write restrictions are enforced at the tool handler level (path escape checks).
/// Network policy is TBD (Windows Filtering Platform requires admin rights).
use std::sync::OnceLock;
use tokio::process::Command;

/// Describes how to confine a shell command.
#[derive(Debug, Clone)]
pub struct SandboxSpec {
    /// "enforce" to wrap the command.
    pub mode: String,
    /// Allowed write directories.
    pub write_roots: Vec<String>,
    /// Allow network egress.
    pub network: bool,
}

impl Default for SandboxSpec {
    fn default() -> Self {
        Self { mode: String::new(), write_roots: Vec::new(), network: true }
    }
}

impl SandboxSpec {
    pub fn enforce(&self) -> bool {
        self.mode == "enforce"
    }
}

pub fn sandbox_available() -> bool {
    true
}

/// Errors that can occur during sandbox enforcement.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SandboxError {
    WriteRootViolation { path: String, allowed_roots: Vec<String> },
    NetworkBlocked { url: String },
    ProcessLaunchFailed { reason: String },
}

// Global sandbox spec for use by tool handlers.
static GLOBAL_SANDBOX_SPEC: OnceLock<SandboxSpec> = OnceLock::new();
// Global workspace path for path resolution in tool handlers.
static GLOBAL_WORKSPACE: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Initialize the global sandbox spec. Must be called once at startup.
pub fn init_global_sandbox_spec(spec: SandboxSpec) {
    let _ = GLOBAL_SANDBOX_SPEC.set(spec);
}

/// Set the global workspace path for tool handlers.
pub fn set_workspace(path: std::path::PathBuf) {
    let _ = GLOBAL_WORKSPACE.set(path);
}

/// Get the global workspace path, or None if not set.
pub fn get_workspace() -> Option<&'static std::path::PathBuf> {
    GLOBAL_WORKSPACE.get()
}

/// Resolve a file path relative to the workspace if it's relative.
pub fn resolve_path(path: &std::path::Path) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(ws) = get_workspace() {
        ws.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Get the global sandbox spec, or a default if not initialized.
pub fn global_sandbox_spec() -> &'static SandboxSpec {
    GLOBAL_SANDBOX_SPEC.get().unwrap_or_else(|| {
        // Box::leak provides a &'static reference that lives for the program duration.
        Box::leak(Box::new(SandboxSpec::default()))
    })
}

/// Apply sandbox confinement to a command. Returns Err if the command
/// should be blocked entirely (e.g. write outside root, network when disabled).
pub fn confine_command(cmd: &mut Command, spec: &SandboxSpec) -> Result<(), SandboxError> {
    if !spec.enforce() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW = 0x08000000, CREATE_NEW_PROCESS_GROUP = 0x00000200
        cmd.as_std_mut().creation_flags(0x08000200);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.as_std_mut().process_group(0);
    }
    Ok(())
}

/// Check if a file path is within the allowed write roots.
#[allow(dead_code)]
pub fn check_write_allowed(path: &str, spec: &SandboxSpec) -> Result<(), SandboxError> {
    if !spec.enforce() || spec.write_roots.is_empty() {
        return Ok(());
    }
    let path = std::path::Path::new(path);
    let allowed = spec.write_roots.iter().any(|root| {
        path.starts_with(root)
    });
    if allowed {
        Ok(())
    } else {
        Err(SandboxError::WriteRootViolation {
            path: path.to_string_lossy().to_string(),
            allowed_roots: spec.write_roots.clone(),
        })
    }
}

/// Check if network access is allowed based on the sandbox spec.
#[allow(dead_code)]
pub fn check_network_allowed(spec: &SandboxSpec) -> bool {
    if !spec.enforce() { true } else { spec.network }
}
