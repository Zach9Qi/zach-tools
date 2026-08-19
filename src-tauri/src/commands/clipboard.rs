//! 剪贴板历史命令：查询、粘贴、复制、删除。
//! 命令层保持薄：取参 -> 调服务 -> 返回。

use tauri::{AppHandle, Runtime, State};

use crate::error::AppError;
use crate::services::clipboard_store::{self as store, ClipboardItem};
use crate::services::{launcher_window, paste};
use crate::state::AppState;

/// 列表默认分页大小
const DEFAULT_LIMIT: i64 = 100;
/// 列表单页上限
const MAX_LIMIT: i64 = 500;

/// 查询剪贴板历史（按最近使用倒序），query 为关键字包含匹配。
#[tauri::command]
pub async fn list_clipboard_items(
    state: State<'_, AppState>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ClipboardItem>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);
    Ok(store::list(state.db(), query.as_deref(), limit, offset).await?)
}

/// 粘贴指定条目：写剪贴板 -> 收起面板 -> 焦点还原到原应用 -> 注入 Ctrl+V。
/// 目前仅支持文本条目。
#[tauri::command]
pub async fn paste_clipboard_item<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    let text = get_text_content(&state, id).await?;

    paste::copy_text_to_clipboard(&state, text).await?;
    launcher_window::hide(&app);
    paste::deliver_paste(state.paste_target()).await?;
    store::touch(state.db(), id).await?;
    Ok(())
}

/// 仅把条目内容复制到系统剪贴板，不执行粘贴（面板保持打开）。
/// 目前仅支持文本条目。
#[tauri::command]
pub async fn copy_clipboard_item(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let text = get_text_content(&state, id).await?;

    paste::copy_text_to_clipboard(&state, text).await?;
    store::touch(state.db(), id).await?;
    Ok(())
}

/// 删除一条历史记录。
#[tauri::command]
pub async fn delete_clipboard_item(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    if !store::delete(state.db(), id).await? {
        return Err(AppError::ItemNotFound);
    }
    Ok(())
}

/// 取指定条目的文本内容：条目不存在报 ItemNotFound，非文本条目报 UnsupportedKind。
async fn get_text_content(state: &AppState, id: i64) -> Result<String, AppError> {
    let item = store::get(state.db(), id)
        .await?
        .ok_or(AppError::ItemNotFound)?;
    item.text_content.ok_or(AppError::UnsupportedKind)
}
