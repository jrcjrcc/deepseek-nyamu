//! DeepWhale 程序入口
//!
//! 根据命令行参数决定运行模式：
//! - 带 `--cli` 参数 → CLI 模式（终端交互/子命令）
//! - 不带参数 → GUI 模式（Tauri 桌面应用）
//!
//! Windows 特殊处理：
//! - GUI 模式：隐藏控制台窗口（`windows_subsystem = "windows"`）
//! - CLI 模式：附加到父进程控制台，以便 println! 正常输出
//!
//! Prevents console window in GUI mode (release build).
//! CLI mode attaches to parent console for println! output.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
/// Windows API：附加到父进程控制台（用于 CLI 模式输出）
unsafe extern "system" {
    fn AttachConsole(dwProcessId: u32) -> i32;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let has_cli_flag = args.iter().any(|a| a == "--cli");

    if has_cli_flag {
        // CLI 模式：附加到父控制台，然后调用 CLI 入口
        #[cfg(windows)]
        unsafe {
            // ATTACH_PARENT_PROCESS = -1 (0xFFFFFFFF)
            // Rust 的 stdout() 是惰性初始化的，
            // 第一次 println! 后才真正获取控制台句柄
            AttachConsole(0xFFFFFFFF);
        }
        nyamuwhale::run_cli_mode();
    } else {
        // GUI 模式：启动 Tauri 桌面窗口
        nyamuwhale::run_gui();
    }
}
