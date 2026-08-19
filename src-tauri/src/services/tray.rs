//! 系统托盘装配：应用常驻后台且窗口不在任务栏显示，
//! 托盘是用户感知进程存在、开合启动器与主动退出的唯一可见入口。

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::services::launcher_window;

/// 托盘图标 id：失焦回调经由它查询图标矩形，判断失焦是否由托盘按下引起
pub const TRAY_ID: &str = "zach-tray";

/// 菜单项 id：打开启动器
const MENU_OPEN: &str = "open-launcher";
/// 菜单项 id：退出应用
const MENU_QUIT: &str = "quit";

/// 创建系统托盘：左键单击开合启动器，右键弹出菜单。
pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // 弹出菜单时窗口必已因失焦收起，菜单项语义固定为「打开」
    let open = MenuItem::with_id(app, MENU_OPEN, "打开启动器", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &quit])?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        // 左键留给开合启动器，菜单只在右键弹出
        .show_menu_on_left_click(false)
        .tooltip("zach-tools")
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_OPEN => launcher_window::show(app),
            // 走 RunEvent::Exit 的优雅退出清理
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                ..
            } = event
            else {
                return;
            };
            match button {
                MouseButton::Left => launcher_window::toggle_from_tray(tray.app_handle()),
                // 托盘引发的失焦不收窗口，弹菜单时在此主动收起
                MouseButton::Right => launcher_window::hide(tray.app_handle()),
                MouseButton::Middle => {}
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.build(app)?;
    Ok(())
}
