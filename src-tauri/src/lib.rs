use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

fn hide_launcher_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.set_ignore_cursor_events(true);
    let _ = window.hide();
}

fn show_launcher_window<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.set_ignore_cursor_events(false);
    let _ = window.show();
    let _ = window.center();
    let _ = window.set_focus();
}

#[tauri::command]
fn hide_launcher<R: Runtime>(app: AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        hide_launcher_window(&window);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

                app.handle()
                    .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

                app.global_shortcut().on_shortcut("alt+enter", |app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    let Some(window) = app.get_webview_window("main") else {
                        return;
                    };

                    let visible = window.is_visible().unwrap_or(false);
                    let focused = window.is_focused().unwrap_or(false);

                    if visible && focused {
                        hide_launcher_window(&window);
                        let _ = app.emit("launcher-close", ());
                    } else {
                        show_launcher_window(&window);
                        let _ = app.emit("launcher-open", ());
                    }
                })?;

                if let Some(window) = app.get_webview_window("main") {
                    let blur_window = window.clone();
                    let blur_app = app.handle().clone();
                    window.on_window_event(move |event| {
                        if matches!(event, tauri::WindowEvent::Focused(false)) {
                            hide_launcher_window(&blur_window);
                            let _ = blur_app.emit("launcher-close", ());
                        }
                    });

                    // The process stays resident while the native window is hidden.
                    hide_launcher_window(&window);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![hide_launcher])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
