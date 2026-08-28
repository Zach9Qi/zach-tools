//! 剪贴板历史命令：查询、粘贴、复制、删除。
//! 命令层保持薄：取参 -> 调服务 -> 返回。

use tauri::{AppHandle, Runtime, State};

use crate::error::AppError;
use crate::services::clipboard_store::{
    self as store, ClipboardItem, ClipboardKind, ListCursor, ListFilter,
};
use crate::services::{launcher_window, paste};
use crate::state::AppState;

/// 列表默认分页大小
const DEFAULT_LIMIT: i64 = 50;
/// 列表单页上限
const MAX_LIMIT: i64 = 1000;

/// 查询剪贴板历史（按最近使用倒序，预览形态）。
/// query 为关键字包含匹配，kind 限定内容类型，favorite_only 只看收藏，三者可叠加；
/// cursor 为 keyset 游标（上一页最后一行的 lastUsedAt + id），缺省返回首页。
#[tauri::command]
pub async fn list_clipboard_items(
    state: State<'_, AppState>,
    query: Option<String>,
    kind: Option<ClipboardKind>,
    favorite_only: Option<bool>,
    limit: Option<i64>,
    cursor: Option<ListCursor>,
) -> Result<Vec<ClipboardItem>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let filter = ListFilter {
        keyword: query.as_deref(),
        kind,
        favorite_only: favorite_only.unwrap_or(false),
    };
    Ok(store::list(state.db(), filter, limit, cursor).await?)
}

/// 设置条目收藏状态；收藏项不参与容量清理。
#[tauri::command]
pub async fn set_clipboard_favorite(
    state: State<'_, AppState>,
    id: i64,
    favorite: bool,
) -> Result<(), AppError> {
    if !store::set_favorite(state.db(), id, favorite).await? {
        return Err(AppError::ItemNotFound);
    }
    Ok(())
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

/// 取指定条目的文本原文：条目不存在报 ItemNotFound，非文本条目报 UnsupportedKind。
async fn get_text_content(state: &AppState, id: i64) -> Result<String, AppError> {
    store::text_content(state.db(), id)
        .await?
        .ok_or(AppError::ItemNotFound)?
        .ok_or(AppError::UnsupportedKind)
}
