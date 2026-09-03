//! 平台相关能力的统一入口：剪贴板监听、前台窗口管理、按键注入与窗口钩子。
//! 仅 Windows 有真实实现，其余平台提供空实现保证编译通过。

/// 监听线程捕获到的一次剪贴板更新。
/// 文本与图片并存时只取文本（见 PRD D4），因此一次更新只会是其中一种。
// 变体只由 Windows 监听线程构造，stub 平台从不发送；非 Windows 下 rustc 会报
// 「variants never constructed」，CI 开了 -D warnings 会被挡下，故仅在该平台放行 dead_code。
#[cfg_attr(not(windows), allow(dead_code))]
pub enum ClipboardCapture {
    /// 剪贴板中的非空文本
    Text(String),
    /// 剪贴板中的位图（arboard 读出的 RGBA8 像素，已持有）
    Image(arboard::ImageData<'static>),
}

#[cfg(windows)]
mod win_input;
#[cfg(windows)]
mod win_monitor;
#[cfg(windows)]
mod win_window;

#[cfg(windows)]
pub use win_input::{focus_window, foreground_window, send_ctrl_v};
#[cfg(windows)]
pub use win_monitor::spawn_monitor;
#[cfg(windows)]
pub use win_window::suppress_alt_sysmenu;

#[cfg(not(windows))]
mod stub;

#[cfg(not(windows))]
pub use stub::{focus_window, foreground_window, send_ctrl_v, spawn_monitor};
