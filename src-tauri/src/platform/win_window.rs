//! 启动器顶层窗口的 Win32 钩子：拦截 Alt 弹出的系统菜单。

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{SC_KEYMENU, WM_NCDESTROY, WM_SYSCOMMAND};

/// 与本模块回调配对的 subclass id，避免和 Tauri / WebView2 自己的 subclass 冲突
const SUBCLASS_ID: usize = 0x5A41_4348; // "ZACH"

/// 在启动器顶层窗口上拦截 `WM_SYSCOMMAND / SC_KEYMENU`。
///
/// 单独按下并松开 Alt（或先 Alt 再 Enter）时，系统会把窗口切入菜单模式；
/// 无边框启动器没有菜单栏，`DefWindowProc` 便弹出系统菜单（还原 / 移动 / 关闭）。
/// Alt+Enter 又是全局唤起键，这个菜单会误伤。吞掉 `SC_KEYMENU` 即可，
/// Alt+F4 等其它系统命令不受影响。失败只打日志，不让启动失败。
pub fn suppress_alt_sysmenu(hwnd: isize) {
    let hwnd = HWND(hwnd as *mut core::ffi::c_void);
    if hwnd.is_invalid() {
        log::warn!("窗口句柄无效，跳过 Alt 系统菜单拦截");
        return;
    }
    let ok = unsafe { SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) };
    if ok.as_bool() {
        log::debug!("已拦截 Alt 触发的窗口系统菜单");
    } else {
        log::warn!("安装 Alt 系统菜单拦截失败");
    }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid: usize,
    _data: usize,
) -> LRESULT {
    // 低 4 位由系统内部使用，比较前必须用 0xFFF0 掩掉
    if msg == WM_SYSCOMMAND && (wparam.0 as u32 & 0xFFF0) == SC_KEYMENU {
        return LRESULT(0);
    }
    if msg == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID);
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
