//! 非 Windows 平台的空实现：剪贴板监听与粘贴注入暂只支持 Windows。

use tokio::sync::mpsc::UnboundedSender;

use super::ClipboardCapture;

pub fn spawn_monitor(_tx: UnboundedSender<ClipboardCapture>) {
    log::warn!("当前平台暂不支持剪贴板监听");
}

pub fn foreground_window() -> Option<isize> {
    None
}

pub fn focus_window(_handle: isize) -> bool {
    false
}

pub fn send_ctrl_v() {}
