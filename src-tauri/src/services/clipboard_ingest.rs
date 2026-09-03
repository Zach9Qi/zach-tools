//! 剪贴板采集链路的入库编排：
//! 平台监听线程 -> 通道 -> 本模块的异步循环（过滤回环与超大内容 -> 入库 -> 清理 -> 通知前端）。
//! 图片分支多一步落盘：像素 hash 查重命中只上浮，未命中才编码写文件再入库；
//! 容量清理淘汰的 image 行由本模块顺手删掉磁盘文件。

use arboard::ImageData;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::platform::{self, ClipboardCapture};
use crate::services::clipboard_store::{self as store, ClipboardItem, ImageRow};
use crate::services::image_store;
use crate::state::AppState;

/// 新条目落库后通知前端刷新的事件名
const EVENT_NEW_ITEM: &str = "clipboard-new-item";

/// 启动采集链路。要求 `AppState` 已注册。
pub fn start<R: Runtime>(app: AppHandle<R>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<ClipboardCapture>();
    platform::spawn_monitor(tx);

    tauri::async_runtime::spawn(async move {
        while let Some(capture) = rx.recv().await {
            if let Err(err) = ingest(&app, capture).await {
                log::warn!("剪贴板内容入库失败: {err}");
            }
        }
    });
}

async fn ingest<R: Runtime>(app: &AppHandle<R>, capture: ClipboardCapture) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let item = match capture {
        ClipboardCapture::Text(text) => ingest_text(&state, text).await?,
        ClipboardCapture::Image(image) => ingest_image(&state, image).await?,
    };
    let Some(item) = item else {
        return Ok(());
    };

    // 文本入库也可能把最旧的 image 行淘汰出容量，统一在这里清理磁盘文件
    let removed = store::prune(state.db(), store::MAX_HISTORY_ITEMS).await?;
    image_store::remove_files(&removed.image_files);
    app.emit(EVENT_NEW_ITEM, &item)?;
    Ok(())
}

/// 文本入库：超长跳过、自写回环跳过，否则 upsert 返回预览形态。
async fn ingest_text(state: &AppState, text: String) -> Result<Option<ClipboardItem>, AppError> {
    if text.len() > store::MAX_TEXT_BYTES {
        log::info!("剪贴板文本超过 {} 字节，跳过记录", store::MAX_TEXT_BYTES);
        return Ok(None);
    }

    let hash = store::hash_text(&text);
    // 本程序自己写入剪贴板（粘贴/复制历史条目）触发的事件，跳过避免回环
    if state.take_self_write_if_matches(&hash) {
        return Ok(None);
    }

    // upsert 返回预览形态，事件载荷不携带原文（超长文本不进 IPC）
    Ok(Some(store::upsert_text(state.db(), &text, &hash).await?))
}

/// 图片入库：hash 与编码都在 `spawn_blocking` 里做，不占 tokio 工作线程。
/// 顺序为 自写守卫 -> 查重上浮 -> 落盘 -> 入库；入库失败会删掉刚写的文件避免孤儿。
async fn ingest_image(
    state: &AppState,
    image: ImageData<'static>,
) -> Result<Option<ClipboardItem>, AppError> {
    let (image, hash) = tauri::async_runtime::spawn_blocking(move || {
        let hash = image_store::hash_image(image.width as u32, image.height as u32, &image.bytes);
        (image, hash)
    })
    .await?;

    if state.take_self_write_if_matches(&hash) {
        return Ok(None);
    }

    // 重复复制同一张图是高频场景：先查重，命中就不再编码写文件
    if let Some(existing) = store::touch_by_hash(state.db(), &hash).await? {
        return Ok(Some(existing));
    }

    let image_dir = state.image_dir().to_path_buf();
    let stored =
        tauri::async_runtime::spawn_blocking(move || image_store::save(&image_dir, &image, &hash))
            .await??;

    let image_path = stored.image_path.to_string_lossy().into_owned();
    let thumbnail_path = stored.thumbnail_path.to_string_lossy().into_owned();
    let row = ImageRow {
        hash: &stored.hash,
        image_path: &image_path,
        thumbnail_path: &thumbnail_path,
        width: i64::from(stored.width),
        height: i64::from(stored.height),
    };
    match store::insert_image(state.db(), row).await {
        Ok(item) => Ok(Some(item)),
        Err(err) => {
            image_store::remove_files(&[image_path, thumbnail_path]);
            Err(err.into())
        }
    }
}
