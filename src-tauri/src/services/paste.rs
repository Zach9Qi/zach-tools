//! 粘贴编排：写剪贴板（带自写标记）、还原焦点到目标窗口并注入 Ctrl+V。

use std::time::Duration;

use crate::error::AppError;
use crate::platform;
use crate::services::clipboard_store as store;
use crate::state::AppState;

/// 焦点切换与按键注入之间的等待，给目标窗口留出激活时间
const FOCUS_SETTLE_DELAY: Duration = Duration::from_millis(50);

/// 把文本写入系统剪贴板，并打上自写标记避免监听回环。
pub async fn copy_text_to_clipboard(state: &AppState, text: String) -> Result<(), AppError> {
    state.mark_self_write(store::hash_text(&text));

    let written = tauri::async_runtime::spawn_blocking(move || {
        arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text))
    })
    .await;

    match written {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => {
            state.clear_self_write();
            Err(err.into())
        }
        Err(err) => {
            state.clear_self_write();
            Err(err.into())
        }
    }
}

/// 把焦点还原到目标窗口并注入 Ctrl+V。
/// 未记录目标窗口时直接向当前前台窗口注入（窗口隐藏后焦点通常已自动回落）。
pub async fn deliver_paste(target: Option<isize>) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        match target {
            Some(handle) => {
                if !platform::focus_window(handle) {
                    log::warn!("还原前台窗口失败，尝试直接注入粘贴");
                }
            }
            None => log::warn!("未记录粘贴目标窗口，直接向当前前台窗口注入"),
        }
        std::thread::sleep(FOCUS_SETTLE_DELAY);
        platform::send_ctrl_v();
    })
    .await?;
    Ok(())
}
