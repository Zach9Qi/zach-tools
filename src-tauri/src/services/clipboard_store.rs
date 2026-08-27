//! 剪贴板历史的存储层：纯 sqlx 读写，不依赖 tauri 类型，便于单元测试。
//! 错误统一返回 `sqlx::Error`，由上层转换为 `AppError`。
//!
//! 列表与事件载荷统一走「预览投影」：`text_content` 截断为预览字符数并附带原文长度，
//! 原文不出库，粘贴/复制按 id 用 [`text_content`] 现取。

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::SqlitePool;

/// 历史容量上限：超出后最旧的非收藏条目会被清理
pub const MAX_HISTORY_ITEMS: i64 = 500;
/// 单条文本上限（字节）：超过则跳过不记录，避免撑爆数据库
pub const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;
/// 跨端传输的文本预览上限（字符）：覆盖列表单行与详情栏的全部显示需求
pub const PREVIEW_MAX_CHARS: i64 = 5000;

/// 剪贴板内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ClipboardKind {
    /// 纯文本
    Text,
    /// 图片：原图与缩略图落盘，库中只存路径
    Image,
    /// 文件/文件夹路径列表
    Files,
}

/// 剪贴板历史条目（跨端传输结构，预览形态）。
/// 三种类型共用一个结构，类型专属字段只在对应 kind 下有值。
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    /// 主键，前端操作（粘贴/复制/删除）时回传
    pub id: i64,
    /// 内容类型
    pub kind: ClipboardKind,
    /// [text] 文本预览（最多 [`PREVIEW_MAX_CHARS`] 字符），原文不出库
    pub text_preview: Option<String>,
    /// [text] 原文总字符数，配合预览判断是否被截断
    pub text_length: Option<i64>,
    /// [image] 原图落盘路径
    pub image_path: Option<String>,
    /// [image] 列表缩略图落盘路径
    pub thumbnail_path: Option<String>,
    /// [image] 原图像素宽度
    pub image_width: Option<i64>,
    /// [image] 原图像素高度
    pub image_height: Option<i64>,
    /// [files] 文件/文件夹绝对路径列表
    pub file_paths: Option<Json<Vec<String>>>,
    /// 是否收藏（收藏项不参与容量清理）
    pub is_favorite: bool,
    /// 首次记录时间（epoch 毫秒）
    pub created_at: i64,
    /// 最近一次复制/使用时间（epoch 毫秒）
    pub last_used_at: i64,
}

/// keyset 分页游标：上一页最后一行的 (last_used_at, id)。
/// 值锚点，不受期间插入/删除影响，翻页不会跳行或重行。
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCursor {
    /// 最后一行的最近使用时间（epoch 毫秒）
    pub last_used_at: i64,
    /// 最后一行的 id，同毫秒时间戳的决胜键
    pub id: i64,
}

/// 预览投影的前半段：接一个绑定参数（预览字符数 [`PREVIEW_MAX_CHARS`]）
const PREVIEW_SELECT_HEAD: &str = "SELECT id, kind, substr(text_content, 1, ";
/// 预览投影的后半段：text_preview 为截断预览，text_length 为原文总字符数
const PREVIEW_SELECT_TAIL: &str = ") AS text_preview, length(text_content) AS text_length, \
     image_path, thumbnail_path, image_width, image_height, file_paths, \
     is_favorite, created_at, last_used_at FROM clipboard_items";

/// 起一个预览投影的查询构造器：`SELECT ...预览列... FROM clipboard_items`
fn preview_query() -> sqlx::QueryBuilder<sqlx::Sqlite> {
    let mut builder = sqlx::QueryBuilder::new(PREVIEW_SELECT_HEAD);
    builder.push_bind(PREVIEW_MAX_CHARS);
    builder.push(PREVIEW_SELECT_TAIL);
    builder
}

/// 计算文本内容的去重 hash（blake3 十六进制）
pub fn hash_text(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

/// 当前 epoch 毫秒时间戳
fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

/// 插入一条文本记录；content_hash 冲突（重复复制）时只把旧条目提到最前。
/// 返回落库后的条目（预览形态，可直接作为事件载荷）。
pub async fn upsert_text(
    pool: &SqlitePool,
    text: &str,
    content_hash: &str,
) -> Result<ClipboardItem, sqlx::Error> {
    let now = now_ms();
    let id: i64 = sqlx::query_scalar(
        r"
        INSERT INTO clipboard_items (kind, text_content, content_hash, created_at, last_used_at)
        VALUES ('text', ?1, ?2, ?3, ?3)
        ON CONFLICT(content_hash) DO UPDATE SET last_used_at = excluded.last_used_at
        RETURNING id
        ",
    )
    .bind(text)
    .bind(content_hash)
    .bind(now)
    .fetch_one(pool)
    .await?;

    fetch_preview(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

/// 按主键取单条记录的预览形态
async fn fetch_preview(pool: &SqlitePool, id: i64) -> Result<Option<ClipboardItem>, sqlx::Error> {
    let mut builder = preview_query();
    builder.push(" WHERE id = ").push_bind(id);
    builder
        .build_query_as::<ClipboardItem>()
        .fetch_optional(pool)
        .await
}

/// 按 (last_used_at DESC, id DESC) 全序分页查询。
/// query 非空时对文本内容做包含匹配（image / files 条目自然被排除在关键字搜索之外）；
/// cursor 非空时返回严格晚于该锚点的下一页（keyset 翻页）。
pub async fn list(
    pool: &SqlitePool,
    query: Option<&str>,
    limit: i64,
    cursor: Option<ListCursor>,
) -> Result<Vec<ClipboardItem>, sqlx::Error> {
    let keyword = query.map(str::trim).filter(|kw| !kw.is_empty());

    let mut builder = preview_query();
    let mut prefix = " WHERE ";
    if let Some(keyword) = keyword {
        builder.push(prefix).push("text_content LIKE ");
        builder.push_bind(format!("%{}%", escape_like(keyword)));
        builder.push(r" ESCAPE '\'");
        prefix = " AND ";
    }
    if let Some(cursor) = cursor {
        builder.push(prefix).push("(last_used_at, id) < (");
        builder.push_bind(cursor.last_used_at);
        builder.push(", ");
        builder.push_bind(cursor.id);
        builder.push(")");
    }
    builder.push(" ORDER BY last_used_at DESC, id DESC LIMIT ");
    builder.push_bind(limit);

    builder
        .build_query_as::<ClipboardItem>()
        .fetch_all(pool)
        .await
}

/// 取条目原文（粘贴/复制历史条目用）。
/// 外层 None 表示条目不存在，内层 None 表示非文本条目。
pub async fn text_content(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<Option<String>>, sqlx::Error> {
    sqlx::query_scalar("SELECT text_content FROM clipboard_items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// 刷新条目的最近使用时间（粘贴/复制历史条目后调用）。
pub async fn touch(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE clipboard_items SET last_used_at = ?1 WHERE id = ?2")
        .bind(now_ms())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 删除单条记录，返回是否确实删掉了。
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM clipboard_items WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// 容量清理：保留最新的 keep 条非收藏记录，收藏项永不清理。
pub async fn prune(pool: &SqlitePool, keep: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        DELETE FROM clipboard_items
        WHERE is_favorite = 0
          AND id NOT IN (
              SELECT id FROM clipboard_items
              WHERE is_favorite = 0
              ORDER BY last_used_at DESC, id DESC
              LIMIT ?1
          )
        ",
    )
    .bind(keep)
    .execute(pool)
    .await?;
    Ok(())
}

/// 转义 LIKE 通配符，配合 `ESCAPE '\'` 使用户输入按字面匹配
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn memory_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("连接内存数据库失败");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("执行 migration 失败");
        pool
    }

    /// 手动指定条目的 last_used_at，构造确定的排序场景
    async fn set_last_used(pool: &SqlitePool, text: &str, value: i64) {
        sqlx::query("UPDATE clipboard_items SET last_used_at = ?1 WHERE text_content = ?2")
            .bind(value)
            .bind(text)
            .execute(pool)
            .await
            .expect("调整时间失败");
    }

    #[tokio::test]
    async fn upsert_dedups_by_hash_and_bumps_last_used() {
        let pool = memory_pool().await;
        let hash = hash_text("hello");

        let first = upsert_text(&pool, "hello", &hash)
            .await
            .expect("首次入库失败");
        assert_eq!(first.kind, ClipboardKind::Text);
        assert_eq!(first.text_preview.as_deref(), Some("hello"));
        assert_eq!(first.text_length, Some(5));

        // 人为把时间调旧，验证重复复制会刷新 last_used_at 而不是新增行
        set_last_used(&pool, "hello", 1).await;

        let second = upsert_text(&pool, "hello", &hash)
            .await
            .expect("再次入库失败");
        assert_eq!(first.id, second.id);
        assert!(second.last_used_at > 1);

        let all = list(&pool, None, 10, None).await.expect("查询失败");
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn prune_removes_oldest_but_keeps_favorites() {
        let pool = memory_pool().await;
        for index in 0..3 {
            let text = format!("item-{index}");
            upsert_text(&pool, &text, &hash_text(&text))
                .await
                .expect("入库失败");
            set_last_used(&pool, &text, index).await;
        }
        sqlx::query("UPDATE clipboard_items SET is_favorite = 1 WHERE text_content = 'item-0'")
            .execute(&pool)
            .await
            .expect("设置收藏失败");

        prune(&pool, 1).await.expect("清理失败");

        let remaining = list(&pool, None, 10, None).await.expect("查询失败");
        // 收藏的 item-0 + 最新的非收藏 item-2
        assert_eq!(remaining.len(), 2);
        assert!(remaining
            .iter()
            .any(|item| item.text_preview.as_deref() == Some("item-0")));
        assert!(remaining
            .iter()
            .any(|item| item.text_preview.as_deref() == Some("item-2")));
    }

    #[tokio::test]
    async fn list_escapes_like_wildcards() {
        let pool = memory_pool().await;
        for text in ["progress 100%", "progress 100x"] {
            upsert_text(&pool, text, &hash_text(text))
                .await
                .expect("入库失败");
        }

        let hits = list(&pool, Some("100%"), 10, None).await.expect("查询失败");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text_preview.as_deref(), Some("progress 100%"));
    }

    #[tokio::test]
    async fn list_paginates_with_keyset_and_survives_deletion() {
        let pool = memory_pool().await;
        for (index, text) in ["a", "b", "c", "d"].iter().enumerate() {
            upsert_text(&pool, text, &hash_text(text))
                .await
                .expect("入库失败");
            set_last_used(&pool, text, index as i64 + 1).await;
        }

        // 第一页：[d, c]
        let page1 = list(&pool, None, 2, None).await.expect("查询失败");
        let texts: Vec<_> = page1.iter().map(|it| it.text_preview.as_deref()).collect();
        assert_eq!(texts, [Some("d"), Some("c")]);

        // 翻页前删掉头部条目 d：offset 分页会因此跳过 b，keyset 不受影响
        assert!(delete(&pool, page1[0].id).await.expect("删除失败"));

        let cursor = ListCursor {
            last_used_at: page1[1].last_used_at,
            id: page1[1].id,
        };
        let page2 = list(&pool, None, 2, Some(cursor)).await.expect("查询失败");
        let texts: Vec<_> = page2.iter().map(|it| it.text_preview.as_deref()).collect();
        assert_eq!(texts, [Some("b"), Some("a")]);
    }

    #[tokio::test]
    async fn list_breaks_same_timestamp_ties_by_id() {
        let pool = memory_pool().await;
        for text in ["tie-1", "tie-2"] {
            upsert_text(&pool, text, &hash_text(text))
                .await
                .expect("入库失败");
            set_last_used(&pool, text, 42).await;
        }

        // 同毫秒并列按 id 倒序决胜：后插入的 tie-2 在前，且跨页不丢不重
        let page1 = list(&pool, None, 1, None).await.expect("查询失败");
        assert_eq!(page1[0].text_preview.as_deref(), Some("tie-2"));

        let cursor = ListCursor {
            last_used_at: page1[0].last_used_at,
            id: page1[0].id,
        };
        let page2 = list(&pool, None, 1, Some(cursor)).await.expect("查询失败");
        assert_eq!(page2[0].text_preview.as_deref(), Some("tie-1"));
    }

    #[tokio::test]
    async fn list_truncates_preview_and_reports_full_length() {
        let pool = memory_pool().await;
        let long_text = "字".repeat(PREVIEW_MAX_CHARS as usize + 100);
        upsert_text(&pool, &long_text, &hash_text(&long_text))
            .await
            .expect("入库失败");

        let items = list(&pool, None, 10, None).await.expect("查询失败");
        let item = &items[0];
        let preview = item.text_preview.as_deref().expect("应有文本预览");
        assert_eq!(preview.chars().count(), PREVIEW_MAX_CHARS as usize);
        assert_eq!(item.text_length, Some(PREVIEW_MAX_CHARS + 100));

        // 原文按 id 现取，不受预览截断影响
        let full = text_content(&pool, item.id)
            .await
            .expect("查询失败")
            .flatten()
            .expect("应有原文");
        assert_eq!(full.chars().count(), PREVIEW_MAX_CHARS as usize + 100);
    }
}
