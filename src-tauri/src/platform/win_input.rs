//! Windows 前台窗口管理与键盘注入：支撑「粘贴回原应用」。

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

/// 取当前前台窗口句柄；没有有效前台窗口时返回 None。
pub fn foreground_window() -> Option<isize> {
    let hwnd = unsafe { GetForegroundWindow() };
    (!hwnd.is_invalid()).then_some(hwnd.0 as isize)
}

/// 把指定窗口带回前台，返回是否成功。
pub fn focus_window(handle: isize) -> bool {
    unsafe { SetForegroundWindow(HWND(handle as *mut core::ffi::c_void)).as_bool() }
}

/// 向当前前台窗口注入 Ctrl+V。
pub fn send_ctrl_v() {
    let inputs = [
        key_event(VK_CONTROL, false),
        key_event(VK_V, false),
        key_event(VK_V, true),
        key_event(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        log::warn!("模拟 Ctrl+V 未完全注入（{sent}/{}）", inputs.len());
    }
}

fn key_event(key: VIRTUAL_KEY, release: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: if release {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
