//! 平台相关能力的统一入口：剪贴板监听、前台窗口管理与按键注入。
//! 仅 Windows 有真实实现，其余平台提供空实现保证编译通过。

/// 监听线程捕获到的一次剪贴板更新
pub struct ClipboardCapture {
    /// 剪贴板中的文本内容
    pub text: String,
}

#[cfg(windows)]
mod win_input;
#[cfg(windows)]
mod win_monitor;

#[cfg(windows)]
pub use win_input::{focus_window, foreground_window, send_ctrl_v};
#[cfg(windows)]
pub use win_monitor::spawn_monitor;

#[cfg(not(windows))]
mod stub;

#[cfg(not(windows))]
pub use stub::{focus_window, foreground_window, send_ctrl_v, spawn_monitor};
