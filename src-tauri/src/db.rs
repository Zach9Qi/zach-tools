use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppError;

/// 数据库文件名（位于应用数据目录下）
const DB_FILE: &str = "zach-tools.db";

/// 初始化 SQLite 连接池并执行内嵌 migration。
pub async fn init_pool<R: Runtime>(app: &AppHandle<R>) -> Result<SqlitePool, AppError> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let options = SqliteConnectOptions::new()
        .filename(data_dir.join(DB_FILE))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
