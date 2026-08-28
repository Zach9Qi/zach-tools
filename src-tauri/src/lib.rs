mod commands;
mod db;
mod error;
mod platform;
mod services;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .setup(|app| {
            // 初始化数据库（建库 + migration）后注册全局状态
            let pool = tauri::async_runtime::block_on(db::init_pool(app.handle()))?;
            app.manage(AppState::new(pool));

            // 启动剪贴板采集链路：平台监听线程 -> 通道 -> 入库循环
            services::clipboard_ingest::start(app.handle().clone());

            #[cfg(desktop)]
            setup_desktop(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::launcher::hide_launcher,
            commands::clipboard::list_clipboard_items,
            commands::clipboard::paste_clipboard_item,
            commands::clipboard::copy_clipboard_item,
            commands::clipboard::delete_clipboard_item,
            commands::clipboard::set_clipboard_favorite,
        ])
        .build(tauri::generate_context!())
        .expect("启动应用失败")
        .run(|app, event| {
            // 优雅退出：进程结束前显式关闭连接池，等在途写入完成并 checkpoint WAL。
            // 监听线程与全局快捷键无需处理，进程退出时由系统自动回收。
            if let tauri::RunEvent::Exit = event {
                let state = app.state::<AppState>();
                tauri::async_runtime::block_on(state.db().close());
                log::info!("数据库连接池已关闭，应用退出");
            }
        });
}

/// 桌面端专属装配：全局快捷键唤起与失焦自动收起。
#[cfg(desktop)]
fn setup_desktop(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    app.handle()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

    // alt+enter 全局快捷键：开合启动器
    app.global_shortcut()
        .on_shortcut("alt+enter", |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            services::launcher_window::toggle(app);
        })?;

    // 失焦自动收起
    if let Some(window) = app.get_webview_window("main") {
        let handle = app.handle().clone();
        window.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::Focused(false)) {
                services::launcher_window::hide_on_blur(&handle);
            }
        });
    }

    // 系统托盘：常驻后台的可见入口，提供开合与退出
    services::tray::setup(app.handle())?;

    // 启动即隐藏，进程常驻后台等待快捷键唤起
    services::launcher_window::init_hidden(app.handle());
    Ok(())
}
