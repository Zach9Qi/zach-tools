//! 启动器窗口相关命令。

use tauri::{AppHandle, Runtime};

use crate::services::launcher_window;

/// 隐藏启动器窗口（前端 Esc 收起时调用）。
#[tauri::command]
pub fn hide_launcher<R: Runtime>(app: AppHandle<R>) {
    launcher_window::hide(&app);
}
