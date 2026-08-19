//! 剪贴板采集链路的入库编排：
//! 平台监听线程 -> 通道 -> 本模块的异步循环（过滤回环与超大内容 -> 入库 -> 清理 -> 通知前端）。

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::platform::{self, ClipboardCapture};
use crate::services::clipboard_store as store;
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
    if capture.text.len() > store::MAX_TEXT_BYTES {
        log::info!("剪贴板文本超过 {} 字节，跳过记录", store::MAX_TEXT_BYTES);
        return Ok(());
    }

    let hash = store::hash_text(&capture.text);
    let state = app.state::<AppState>();

    // 本程序自己写入剪贴板（粘贴/复制历史条目）触发的事件，跳过避免回环
    if state.take_self_write_if_matches(&hash) {
        return Ok(());
    }

    let item = store::upsert_text(state.db(), &capture.text, &hash).await?;
    store::prune(state.db(), store::MAX_HISTORY_ITEMS).await?;
    app.emit(EVENT_NEW_ITEM, &item)?;
    Ok(())
}
