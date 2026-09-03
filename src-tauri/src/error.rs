/// 全局统一错误类型：可失败命令一律返回 `Result<T, AppError>`，
/// 序列化时输出用户可读的中文文案供前端直接展示。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("数据库迁移失败: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("剪贴板访问失败: {0}")]
    Clipboard(#[from] arboard::Error),

    #[error("没有找到对应的剪贴板记录")]
    ItemNotFound,

    #[error("暂不支持对该类型条目执行此操作")]
    UnsupportedKind,

    #[error("图片文件读取失败: {0}")]
    ImageFile(#[from] image::ImageError),

    #[error("输入输出错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("系统错误: {0}")]
    Tauri(#[from] tauri::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
