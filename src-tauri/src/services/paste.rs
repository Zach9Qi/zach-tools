//! 粘贴编排：写剪贴板（带自写标记）、还原焦点到目标窗口并注入 Ctrl+V。
//! 文本与图片共用同一套自写标记协议：hash 与采集侧同源，监听到自己写入的事件即可吞掉。

use std::path::Path;
use std::time::Duration;

use arboard::ImageData;

use crate::error::AppError;
use crate::platform;
use crate::services::clipboard_store::{self as store, ClipContent};
use crate::services::image_store;
use crate::state::AppState;

/// 焦点切换与按键注入之间的等待，给目标窗口留出激活时间
const FOCUS_SETTLE_DELAY: Duration = Duration::from_millis(50);

/// 把条目内容写入系统剪贴板，并打上自写标记避免监听回环。
/// 图片先从落盘原图解码 RGBA 再算 hash，文件缺失或损坏返回 [`AppError::ImageFile`]。
pub async fn copy_to_clipboard(state: &AppState, content: ClipContent) -> Result<(), AppError> {
    match content {
        ClipContent::Text(text) => {
            state.mark_self_write(store::hash_text(&text));
            write_clipboard(state, move |clipboard| clipboard.set_text(text)).await
        }
        ClipContent::Image { path } => {
            let (image, hash) = tauri::async_runtime::spawn_blocking(move || {
                let image = image_store::load_rgba(Path::new(&path))?;
                let hash =
                    image_store::hash_image(image.width as u32, image.height as u32, &image.bytes);
                Ok::<(ImageData<'static>, String), AppError>((image, hash))
            })
            .await??;
            state.mark_self_write(hash);
            write_clipboard(state, move |clipboard| clipboard.set_image(image)).await
        }
    }
}

/// 在阻塞线程里打开剪贴板执行写入；任一环节失败都回滚自写标记，避免误吞下一次真实复制。
async fn write_clipboard<F>(state: &AppState, write: F) -> Result<(), AppError>
where
    F: FnOnce(&mut arboard::Clipboard) -> Result<(), arboard::Error> + Send + 'static,
{
    let written = tauri::async_runtime::spawn_blocking(move || {
        arboard::Clipboard::new().and_then(|mut clipboard| write(&mut clipboard))
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
