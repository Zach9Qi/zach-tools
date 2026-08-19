//! 剪贴板历史的存储层：纯 sqlx 读写，不依赖 tauri 类型，便于单元测试。
//! 错误统一返回 `sqlx::Error`，由上层转换为 `AppError`。

use serde::Serialize;
use sqlx::types::Json;
use sqlx::SqlitePool;

/// 历史容量上限：超出后最旧的非收藏条目会被清理
pub const MAX_HISTORY_ITEMS: i64 = 500;
/// 单条文本上限（字节）：超过则跳过不记录，避免撑爆数据库
pub const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;

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

/// 剪贴板历史条目（跨端传输结构）。
/// 三种类型共用一个结构，类型专属字段只在对应 kind 下有值。
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    /// 主键，前端操作（粘贴/复制/删除）时回传
    pub id: i64,
    /// 内容类型
    pub kind: ClipboardKind,
    /// [text] 文本内容
    pub text_content: Option<String>,
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
/// 返回落库后的条目。
pub async fn upsert_text(
    pool: &SqlitePool,
    text: &str,
    content_hash: &str,
) -> Result<ClipboardItem, sqlx::Error> {
    let now = now_ms();
    sqlx::query_as::<_, ClipboardItem>(
        r"
        INSERT INTO clipboard_items (kind, text_content, content_hash, created_at, last_used_at)
        VALUES ('text', ?1, ?2, ?3, ?3)
        ON CONFLICT(content_hash) DO UPDATE SET last_used_at = excluded.last_used_at
        RETURNING *
        ",
    )
    .bind(text)
    .bind(content_hash)
    .bind(now)
    .fetch_one(pool)
    .await
}

/// 按最近使用时间倒序分页查询。
/// query 非空时对文本内容做包含匹配（image / files 条目自然被排除在关键字搜索之外）。
pub async fn list(
    pool: &SqlitePool,
    query: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ClipboardItem>, sqlx::Error> {
    let keyword = query.map(str::trim).filter(|kw| !kw.is_empty());
    match keyword {
        Some(keyword) => {
            let pattern = format!("%{}%", escape_like(keyword));
            sqlx::query_as::<_, ClipboardItem>(
                r"
                SELECT * FROM clipboard_items
                WHERE text_content LIKE ?1 ESCAPE '\'
                ORDER BY last_used_at DESC
                LIMIT ?2 OFFSET ?3
                ",
            )
            .bind(pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, ClipboardItem>(
                r"
                SELECT * FROM clipboard_items
                ORDER BY last_used_at DESC
                LIMIT ?1 OFFSET ?2
                ",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
    }
}

/// 按主键取单条记录。
pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<ClipboardItem>, sqlx::Error> {
    sqlx::query_as::<_, ClipboardItem>("SELECT * FROM clipboard_items WHERE id = ?1")
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
              ORDER BY last_used_at DESC
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

    #[tokio::test]
    async fn upsert_dedups_by_hash_and_bumps_last_used() {
        let pool = memory_pool().await;
        let hash = hash_text("hello");

        let first = upsert_text(&pool, "hello", &hash)
            .await
            .expect("首次入库失败");
        assert_eq!(first.kind, ClipboardKind::Text);
        assert_eq!(first.text_content.as_deref(), Some("hello"));

        // 人为把时间调旧，验证重复复制会刷新 last_used_at 而不是新增行
        sqlx::query("UPDATE clipboard_items SET last_used_at = 1 WHERE id = ?1")
            .bind(first.id)
            .execute(&pool)
            .await
            .expect("调整时间失败");

        let second = upsert_text(&pool, "hello", &hash)
            .await
            .expect("再次入库失败");
        assert_eq!(first.id, second.id);
        assert!(second.last_used_at > 1);

        let all = list(&pool, None, 10, 0).await.expect("查询失败");
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
            // 保证 last_used_at 严格递增，避免同毫秒排序不稳定
            sqlx::query("UPDATE clipboard_items SET last_used_at = ?1 WHERE text_content = ?2")
                .bind(index)
                .bind(&text)
                .execute(&pool)
                .await
                .expect("调整时间失败");
        }
        sqlx::query("UPDATE clipboard_items SET is_favorite = 1 WHERE text_content = 'item-0'")
            .execute(&pool)
            .await
            .expect("设置收藏失败");

        prune(&pool, 1).await.expect("清理失败");

        let remaining = list(&pool, None, 10, 0).await.expect("查询失败");
        // 收藏的 item-0 + 最新的非收藏 item-2
        assert_eq!(remaining.len(), 2);
        assert!(remaining
            .iter()
            .any(|item| item.text_content.as_deref() == Some("item-0")));
        assert!(remaining
            .iter()
            .any(|item| item.text_content.as_deref() == Some("item-2")));
    }

    #[tokio::test]
    async fn list_escapes_like_wildcards() {
        let pool = memory_pool().await;
        for text in ["progress 100%", "progress 100x"] {
            upsert_text(&pool, text, &hash_text(text))
                .await
                .expect("入库失败");
        }

        let hits = list(&pool, Some("100%"), 10, 0).await.expect("查询失败");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text_content.as_deref(), Some("progress 100%"));
    }
}
