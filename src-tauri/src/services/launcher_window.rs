//! 启动器窗口的开合编排：维护粘贴目标窗口的记录，并向前端广播开合事件。

use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::platform;
use crate::state::AppState;

/// 主窗口 label（与 tauri.conf.json 保持一致）
const MAIN_WINDOW: &str = "main";
/// 窗口唤起事件
const EVENT_OPEN: &str = "launcher-open";
/// 窗口收起事件
const EVENT_CLOSE: &str = "launcher-close";

/// 展示启动器：先记录当前前台窗口作为后续粘贴目标，再显示并聚焦。
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        state.remember_paste_target(platform::foreground_window());
    }

    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.set_ignore_cursor_events(false);
    let _ = window.show();
    let _ = window.center();
    let _ = window.set_focus();
    let _ = app.emit(EVENT_OPEN, ());
}

/// 收起启动器并通知前端复位状态。
pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = main_window(app) else {
        return;
    };
    hide_window(&window);
    let _ = app.emit(EVENT_CLOSE, ());
}

/// 全局快捷键的开合切换。
pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = main_window(app) else {
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    if visible && focused {
        hide(app);
    } else {
        show(app);
    }
}

/// 应用启动时的初始收纳：只隐藏窗口，不广播事件（此时前端尚未加载）。
pub fn init_hidden<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = main_window(app) {
        hide_window(&window);
    }
}

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(MAIN_WINDOW)
}

fn hide_window<R: Runtime>(window: &WebviewWindow<R>) {
    // 隐藏期间忽略鼠标事件，防止透明窗口残留的命中区域挡住桌面点击
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.hide();
}
