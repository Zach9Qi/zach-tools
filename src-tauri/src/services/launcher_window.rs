//! 启动器窗口的开合编排：维护粘贴目标窗口的记录，并向前端广播开合事件。

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Runtime, WebviewWindow};

use crate::platform;
use crate::services::tray;
use crate::state::AppState;

/// 主窗口 label（与 tauri.conf.json 保持一致）
const MAIN_WINDOW: &str = "main";
/// 窗口唤起事件
const EVENT_OPEN: &str = "launcher-open";
/// 窗口收起事件
const EVENT_CLOSE: &str = "launcher-close";

/// 展示启动器：先记录当前前台窗口作为后续粘贴目标，再定位、显示并聚焦。
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<AppState>() {
        state.remember_paste_target(platform::foreground_window());
    }

    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.set_ignore_cursor_events(false);
    position_anchored(&window);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit(EVENT_OPEN, ());
}

/// 把窗口摆到启动器的惯例位置：水平居中，顶边固定在工作区高度的 1/4 处
/// （Flow Launcher / PowerToys Run / Spotlight 同款摆法）。
/// 顶边只取决于工作区，不依赖内容尺寸；内容增高时窗口从这条顶边向下生长，
/// 搜索框在任何状态切换中都不移动。
fn position_anchored<R: Runtime>(window: &WebviewWindow<R>) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let work = monitor.work_area();
    let x = f64::from(work.position.x) + (f64::from(work.size.width) - f64::from(size.width)) / 2.0;
    let y = f64::from(work.position.y) + f64::from(work.size.height) / 4.0;
    // 工作区比窗口还窄时贴住左缘，不让面板顶出屏幕
    let x = x.max(f64::from(work.position.x));
    let _ = window.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
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

/// 失焦自动收起。鼠标正悬于本应用托盘图标时跳过：
/// 该失焦由托盘按下引起，窗口保持原状，开合决策交给随后到达的托盘点击事件，
/// 这样点击事件看到的是窗口的真实可见状态，无需事后猜测失焦原因。
pub fn hide_on_blur<R: Runtime>(app: &AppHandle<R>) {
    if cursor_on_tray(app) {
        return;
    }
    hide(app);
}

/// 托盘左键点击的开合切换。
///
/// 托盘引发的失焦不会收起窗口（见 [`hide_on_blur`]），因此此处的可见性
/// 是点击前的真实状态：开着就收起，关着就唤起。
pub fn toggle_from_tray<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = main_window(app) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide(app);
    } else {
        show(app);
    }
}

/// 当前鼠标是否悬于本应用的托盘图标上。任一信息拿不到时按「不在」处理，
/// 退化为普通失焦收起。
fn cursor_on_tray<R: Runtime>(app: &AppHandle<R>) -> bool {
    let Some(tray) = app.tray_by_id(tray::TRAY_ID) else {
        return false;
    };
    let Ok(Some(rect)) = tray.rect() else {
        return false;
    };
    let Ok(cursor) = app.cursor_position() else {
        return false;
    };
    rect_contains(&rect, cursor.x, cursor.y)
}

/// 判断屏幕坐标是否落在矩形内（托盘 rect 与鼠标位置均为物理像素坐标）
fn rect_contains(rect: &tauri::Rect, x: f64, y: f64) -> bool {
    let (left, top) = match rect.position {
        tauri::Position::Physical(p) => (f64::from(p.x), f64::from(p.y)),
        tauri::Position::Logical(p) => (p.x, p.y),
    };
    let (width, height) = match rect.size {
        tauri::Size::Physical(s) => (f64::from(s.width), f64::from(s.height)),
        tauri::Size::Logical(s) => (s.width, s.height),
    };
    x >= left && x < left + width && y >= top && y < top + height
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
