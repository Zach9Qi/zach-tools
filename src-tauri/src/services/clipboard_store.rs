//! 剪贴板历史的存储层：纯 sqlx 读写，不依赖 tauri 类型，便于单元测试。
//! 错误统一返回 `sqlx::Error`，由上层转换为 `AppError`。
//!
//! 列表与事件载荷统一走「预览投影」：`text_content` 截断为预览字符数并附带原文长度，
//! 原文不出库，粘贴/复制按 id 用 [`content`] 现取。图片二进制落盘，库中只存路径；
//! 删除类操作通过 [`Removed`] 把被删行的图片路径交给服务层清理磁盘，存储层本身不碰 fs。

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
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

/// 列表过滤条件：分页之外的全部筛选维度，各维度可叠加（AND 语义）
#[derive(Debug, Clone, Copy, Default)]
pub struct ListFilter<'a> {
    /// 关键字，对文本内容做包含匹配；None 或空白不过滤
    pub keyword: Option<&'a str>,
    /// 限定内容类型；None 不限
    pub kind: Option<ClipboardKind>,
    /// 只看收藏
    pub favorite_only: bool,
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

/// 写入 image 行所需的字段（路径已由 image_store 落盘）
#[derive(Debug, Clone, Copy)]
pub struct ImageRow<'a> {
    /// 像素 hash（全局去重键）
    pub hash: &'a str,
    /// 原图落盘路径
    pub image_path: &'a str,
    /// 缩略图落盘路径
    pub thumbnail_path: &'a str,
    /// 原图像素宽度
    pub width: i64,
    /// 原图像素高度
    pub height: i64,
}

/// 按 id 现取的条目内容（粘贴/复制用），只覆盖能写回剪贴板的类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipContent {
    /// 文本原文
    Text(String),
    /// 图片：原图落盘路径，由调用方读文件解码
    Image { path: String },
}

/// 删除操作波及的磁盘文件：服务层据此清理，避免孤儿图片
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Removed {
    /// 被删 image 行的原图与缩略图路径（文本行不产生路径）
    pub image_files: Vec<String>,
}

impl Removed {
    /// 从 `RETURNING image_path, thumbnail_path` 的行集收集非空路径
    fn from_rows(rows: Vec<(Option<String>, Option<String>)>) -> Self {
        let image_files = rows
            .into_iter()
            .flat_map(|(image, thumb)| [image, thumb])
            .flatten()
            .collect();
        Self { image_files }
    }
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

/// 插入一条图片记录；content_hash 冲突时只刷新 last_used_at（兜底并发竞态，
/// 常规去重应先走 [`touch_by_hash`] 避免白编码）。返回落库后的预览形态。
pub async fn insert_image(
    pool: &SqlitePool,
    row: ImageRow<'_>,
) -> Result<ClipboardItem, sqlx::Error> {
    let now = now_ms();
    let id: i64 = sqlx::query_scalar(
        r"
        INSERT INTO clipboard_items
            (kind, image_path, thumbnail_path, image_width, image_height, content_hash, created_at, last_used_at)
        VALUES ('image', ?1, ?2, ?3, ?4, ?5, ?6, ?6)
        ON CONFLICT(content_hash) DO UPDATE SET last_used_at = excluded.last_used_at
        RETURNING id
        ",
    )
    .bind(row.image_path)
    .bind(row.thumbnail_path)
    .bind(row.width)
    .bind(row.height)
    .bind(row.hash)
    .bind(now)
    .fetch_one(pool)
    .await?;

    fetch_preview(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

/// 按内容 hash 查重：命中则刷新 last_used_at 并返回预览形态，未命中返回 None。
/// 重复复制图片时先走这里，避免重新编码与写文件。
pub async fn touch_by_hash(
    pool: &SqlitePool,
    hash: &str,
) -> Result<Option<ClipboardItem>, sqlx::Error> {
    let id: Option<i64> = sqlx::query_scalar(
        "UPDATE clipboard_items SET last_used_at = ?1 WHERE content_hash = ?2 RETURNING id",
    )
    .bind(now_ms())
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    match id {
        Some(id) => fetch_preview(pool, id).await,
        None => Ok(None),
    }
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
/// filter 各维度可叠加：关键字对文本内容做包含匹配（image / files 条目自然被排除在关键字搜索之外）、
/// 内容类型、只看收藏；cursor 非空时返回严格晚于该锚点的下一页（keyset 翻页）。
pub async fn list(
    pool: &SqlitePool,
    filter: ListFilter<'_>,
    limit: i64,
    cursor: Option<ListCursor>,
) -> Result<Vec<ClipboardItem>, sqlx::Error> {
    let keyword = filter.keyword.map(str::trim).filter(|kw| !kw.is_empty());

    let mut builder = preview_query();
    let mut prefix = " WHERE ";
    if let Some(keyword) = keyword {
        builder.push(prefix).push("text_content LIKE ");
        builder.push_bind(format!("%{}%", escape_like(keyword)));
        builder.push(r" ESCAPE '\'");
        prefix = " AND ";
    }
    if let Some(kind) = filter.kind {
        builder.push(prefix).push("kind = ").push_bind(kind);
        prefix = " AND ";
    }
    if filter.favorite_only {
        builder.push(prefix).push("is_favorite = 1");
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

/// 取条目可写回剪贴板的内容（粘贴/复制历史条目用）。
/// 外层 None 表示条目不存在，内层 None 表示类型不支持写回（如 files）。
pub async fn content(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<Option<ClipContent>>, sqlx::Error> {
    let row: Option<(ClipboardKind, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT kind, text_content, image_path FROM clipboard_items WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|(kind, text, image_path)| match kind {
        ClipboardKind::Text => text.map(ClipContent::Text),
        ClipboardKind::Image => image_path.map(|path| ClipContent::Image { path }),
        ClipboardKind::Files => None,
    }))
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

/// 设置条目收藏状态，返回是否确有该条目。收藏项不参与容量清理。
pub async fn set_favorite(pool: &SqlitePool, id: i64, favorite: bool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE clipboard_items SET is_favorite = ?1 WHERE id = ?2")
        .bind(favorite)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// 删除单条记录：None 表示条目不存在，Some 携带需要服务层清理的图片文件路径。
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Option<Removed>, sqlx::Error> {
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "DELETE FROM clipboard_items WHERE id = ?1 RETURNING image_path, thumbnail_path",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| Removed::from_rows(vec![row])))
}

/// 容量清理：保留最新的 keep 条非收藏记录，收藏项永不清理。
/// 返回被清理行涉及的图片文件路径，由服务层删除磁盘文件。
pub async fn prune(pool: &SqlitePool, keep: i64) -> Result<Removed, sqlx::Error> {
    let rows: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
        r"
        DELETE FROM clipboard_items
        WHERE is_favorite = 0
          AND id NOT IN (
              SELECT id FROM clipboard_items
              WHERE is_favorite = 0
              ORDER BY last_used_at DESC, id DESC
              LIMIT ?1
          )
        RETURNING image_path, thumbnail_path
        ",
    )
    .bind(keep)
    .fetch_all(pool)
    .await?;
    Ok(Removed::from_rows(rows))
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

    /// 以固定尺寸 640×480 与 `images/<hash>[.thumb].png` 路径写入一条 image 行
    async fn insert_sample_image(pool: &SqlitePool, hash: &str) -> ClipboardItem {
        let image_path = format!("images/{hash}.png");
        let thumbnail_path = format!("images/{hash}.thumb.png");
        insert_image(
            pool,
            ImageRow {
                hash,
                image_path: &image_path,
                thumbnail_path: &thumbnail_path,
                width: 640,
                height: 480,
            },
        )
        .await
        .expect("图片入库失败")
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

        let all = list(&pool, ListFilter::default(), 10, None)
            .await
            .expect("查询失败");
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

        let remaining = list(&pool, ListFilter::default(), 10, None)
            .await
            .expect("查询失败");
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

        let filter = ListFilter {
            keyword: Some("100%"),
            ..ListFilter::default()
        };
        let hits = list(&pool, filter, 10, None).await.expect("查询失败");
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
        let page1 = list(&pool, ListFilter::default(), 2, None)
            .await
            .expect("查询失败");
        let texts: Vec<_> = page1.iter().map(|it| it.text_preview.as_deref()).collect();
        assert_eq!(texts, [Some("d"), Some("c")]);

        // 翻页前删掉头部条目 d：offset 分页会因此跳过 b，keyset 不受影响
        assert!(delete(&pool, page1[0].id)
            .await
            .expect("删除失败")
            .is_some());

        let cursor = ListCursor {
            last_used_at: page1[1].last_used_at,
            id: page1[1].id,
        };
        let page2 = list(&pool, ListFilter::default(), 2, Some(cursor))
            .await
            .expect("查询失败");
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
        let page1 = list(&pool, ListFilter::default(), 1, None)
            .await
            .expect("查询失败");
        assert_eq!(page1[0].text_preview.as_deref(), Some("tie-2"));

        let cursor = ListCursor {
            last_used_at: page1[0].last_used_at,
            id: page1[0].id,
        };
        let page2 = list(&pool, ListFilter::default(), 1, Some(cursor))
            .await
            .expect("查询失败");
        assert_eq!(page2[0].text_preview.as_deref(), Some("tie-1"));
    }

    #[tokio::test]
    async fn list_filters_by_kind_and_favorite() {
        let pool = memory_pool().await;
        for text in ["alpha", "beta"] {
            upsert_text(&pool, text, &hash_text(text))
                .await
                .expect("入库失败");
        }
        insert_sample_image(&pool, "image-hash").await;

        let text_only = ListFilter {
            kind: Some(ClipboardKind::Text),
            ..ListFilter::default()
        };
        let texts = list(&pool, text_only, 10, None).await.expect("查询失败");
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().all(|item| item.kind == ClipboardKind::Text));

        let image_only = ListFilter {
            kind: Some(ClipboardKind::Image),
            ..ListFilter::default()
        };
        let images = list(&pool, image_only, 10, None).await.expect("查询失败");
        assert_eq!(images.len(), 1);
        assert_eq!(
            images[0].image_path.as_deref(),
            Some("images/image-hash.png")
        );

        // 收藏 alpha 后，favorite_only 只回收藏条目
        let alpha_id = texts
            .iter()
            .find(|item| item.text_preview.as_deref() == Some("alpha"))
            .expect("应有 alpha")
            .id;
        assert!(set_favorite(&pool, alpha_id, true).await.expect("收藏失败"));

        let favorite_only = ListFilter {
            favorite_only: true,
            ..ListFilter::default()
        };
        let favorites = list(&pool, favorite_only, 10, None)
            .await
            .expect("查询失败");
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].text_preview.as_deref(), Some("alpha"));
        assert!(favorites[0].is_favorite);

        // 取消收藏后列表清空；不存在的 id 返回 false
        assert!(set_favorite(&pool, alpha_id, false)
            .await
            .expect("取消收藏失败"));
        let favorites = list(&pool, favorite_only, 10, None)
            .await
            .expect("查询失败");
        assert!(favorites.is_empty());
        assert!(!set_favorite(&pool, 9999, true).await.expect("调用失败"));
    }

    #[tokio::test]
    async fn list_truncates_preview_and_reports_full_length() {
        let pool = memory_pool().await;
        let long_text = "字".repeat(PREVIEW_MAX_CHARS as usize + 100);
        upsert_text(&pool, &long_text, &hash_text(&long_text))
            .await
            .expect("入库失败");

        let items = list(&pool, ListFilter::default(), 10, None)
            .await
            .expect("查询失败");
        let item = &items[0];
        let preview = item.text_preview.as_deref().expect("应有文本预览");
        assert_eq!(preview.chars().count(), PREVIEW_MAX_CHARS as usize);
        assert_eq!(item.text_length, Some(PREVIEW_MAX_CHARS + 100));

        // 原文按 id 现取，不受预览截断影响
        let full = content(&pool, item.id)
            .await
            .expect("查询失败")
            .flatten()
            .expect("应有原文");
        let ClipContent::Text(full) = full else {
            panic!("文本条目应返回 Text 内容");
        };
        assert_eq!(full.chars().count(), PREVIEW_MAX_CHARS as usize + 100);
    }

    #[tokio::test]
    async fn insert_image_dedups_by_hash_and_touch_by_hash_bumps() {
        let pool = memory_pool().await;

        // 未命中的 hash 查重返回 None，不应产生任何行
        assert!(touch_by_hash(&pool, "img-a")
            .await
            .expect("查重失败")
            .is_none());

        let first = insert_sample_image(&pool, "img-a").await;
        assert_eq!(first.kind, ClipboardKind::Image);
        assert_eq!(first.image_path.as_deref(), Some("images/img-a.png"));
        assert_eq!(
            first.thumbnail_path.as_deref(),
            Some("images/img-a.thumb.png")
        );
        assert_eq!(
            (first.image_width, first.image_height),
            (Some(640), Some(480))
        );
        assert!(first.text_preview.is_none());
        assert!(first.text_length.is_none());

        // 人为调旧后查重命中：同一 id 上浮，不新增行
        sqlx::query("UPDATE clipboard_items SET last_used_at = 1 WHERE content_hash = 'img-a'")
            .execute(&pool)
            .await
            .expect("调整时间失败");
        let touched = touch_by_hash(&pool, "img-a")
            .await
            .expect("查重失败")
            .expect("应命中已有图片");
        assert_eq!(touched.id, first.id);
        assert!(touched.last_used_at > 1);

        // 并发竞态兜底：同 hash 再次 insert 也只是刷新时间
        let again = insert_sample_image(&pool, "img-a").await;
        assert_eq!(again.id, first.id);

        let all = list(&pool, ListFilter::default(), 10, None)
            .await
            .expect("查询失败");
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn content_distinguishes_missing_unsupported_and_kinds() {
        let pool = memory_pool().await;
        let text = upsert_text(&pool, "hello", &hash_text("hello"))
            .await
            .expect("文本入库失败");
        let image = insert_sample_image(&pool, "img-c").await;
        // files 尚无入库路径，手插一行验证「存在但不支持写回」分支
        let files_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO clipboard_items (kind, file_paths, content_hash, created_at, last_used_at)
            VALUES ('files', '["a.txt"]', 'files-hash', 1, 1)
            RETURNING id
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("插入 files 行失败");

        assert_eq!(
            content(&pool, text.id).await.expect("查询失败"),
            Some(Some(ClipContent::Text("hello".into())))
        );
        assert_eq!(
            content(&pool, image.id).await.expect("查询失败"),
            Some(Some(ClipContent::Image {
                path: "images/img-c.png".into()
            }))
        );
        assert_eq!(
            content(&pool, files_id).await.expect("查询失败"),
            Some(None)
        );
        assert_eq!(content(&pool, 9999).await.expect("查询失败"), None);
    }

    #[tokio::test]
    async fn delete_returns_image_paths_only_for_image_rows() {
        let pool = memory_pool().await;
        let text = upsert_text(&pool, "plain", &hash_text("plain"))
            .await
            .expect("文本入库失败");
        let image = insert_sample_image(&pool, "img-d").await;

        // 文本行：删除成功但没有文件要清理
        assert_eq!(
            delete(&pool, text.id).await.expect("删除失败"),
            Some(Removed::default())
        );
        // 图片行：原图与缩略图路径都要交给服务层清理
        assert_eq!(
            delete(&pool, image.id).await.expect("删除失败"),
            Some(Removed {
                image_files: vec!["images/img-d.png".into(), "images/img-d.thumb.png".into()]
            })
        );
        // 不存在的条目
        assert_eq!(delete(&pool, image.id).await.expect("删除失败"), None);
    }

    #[tokio::test]
    async fn prune_returns_paths_of_evicted_images_and_spares_favorites() {
        let pool = memory_pool().await;
        // 时间从旧到新：old-image(1) < fav-image(2) < text(3) < new-image(4)
        let old = insert_sample_image(&pool, "img-old").await;
        let fav = insert_sample_image(&pool, "img-fav").await;
        upsert_text(&pool, "text", &hash_text("text"))
            .await
            .expect("文本入库失败");
        let new = insert_sample_image(&pool, "img-new").await;
        for (id, value) in [(old.id, 1), (fav.id, 2), (new.id, 4)] {
            sqlx::query("UPDATE clipboard_items SET last_used_at = ?1 WHERE id = ?2")
                .bind(value)
                .bind(id)
                .execute(&pool)
                .await
                .expect("调整时间失败");
        }
        set_last_used(&pool, "text", 3).await;
        assert!(set_favorite(&pool, fav.id, true).await.expect("收藏失败"));

        // 只保留 1 条非收藏：text 与 old-image 被清理，收藏的 fav-image 幸免
        let removed = prune(&pool, 1).await.expect("清理失败");
        assert_eq!(
            removed.image_files,
            vec![
                "images/img-old.png".to_string(),
                "images/img-old.thumb.png".to_string()
            ]
        );

        let remaining = list(&pool, ListFilter::default(), 10, None)
            .await
            .expect("查询失败");
        let ids: Vec<_> = remaining.iter().map(|item| item.id).collect();
        assert_eq!(ids, vec![new.id, fav.id]);
    }
}
