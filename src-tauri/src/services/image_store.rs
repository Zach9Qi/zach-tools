//! 剪贴板图片的落盘服务：像素 hash、原图 PNG 编码、缩略图生成、文件读写与删除。
//! 只依赖 fs 与 image crate，不依赖 tauri 类型；调用方需自行放进 `spawn_blocking`。
//!
//! 文件布局：`<dir>/<hash>.png`（原图，无损）与 `<dir>/<hash>.thumb.png`（列表缩略图）。
//! [`hash_image`] 是图片 hash 的唯一定义处，采集侧与粘贴侧共用，保证自写标记能命中。

use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

use arboard::ImageData;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{
    ExtendedColorType, ImageBuffer, ImageEncoder, ImageError, ImageFormat, ImageReader, Rgba,
};

/// 图片目录名（位于 `app_local_data_dir()` 下，与 SQLite 库同根），与 `tauri.conf.json5` 的 assetProtocol scope 同步
pub const IMAGE_DIR_NAME: &str = "clipboard-images";
/// 缩略图长边上限（像素）：只缩不放
pub const THUMBNAIL_MAX_EDGE: u32 = 200;

/// 一张已落盘图片的元数据，供存储层写入 image 行
#[derive(Debug, Clone)]
pub struct StoredImage {
    /// 像素 hash（blake3 十六进制），全局去重键
    pub hash: String,
    /// 原图 PNG 路径
    pub image_path: PathBuf,
    /// 缩略图 PNG 路径
    pub thumbnail_path: PathBuf,
    /// 原图像素宽度
    pub width: u32,
    /// 原图像素高度
    pub height: u32,
}

/// 计算图片的去重 hash：`blake3(宽 ‖ 高 ‖ RGBA 字节)`，宽高按 u32 小端序参与。
/// 宽高参与是为了区分像素字节相同但排布不同的图片。
pub fn hash_image(width: u32, height: u32, rgba: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&width.to_le_bytes());
    hasher.update(&height.to_le_bytes());
    hasher.update(rgba);
    hasher.finalize().to_hex().to_string()
}

/// 按 hash 推导原图与缩略图路径：`(<dir>/<hash>.png, <dir>/<hash>.thumb.png)`
pub fn paths_for(dir: &Path, hash: &str) -> (PathBuf, PathBuf) {
    (
        dir.join(format!("{hash}.png")),
        dir.join(format!("{hash}.thumb.png")),
    )
}

/// 把 RGBA 像素编码为原图 PNG 与缩略图 PNG 写入 dir。
/// 任一步失败会删掉本次已写出的文件再返回错误，避免留下半成品。
pub fn save(dir: &Path, image: &ImageData<'_>, hash: &str) -> io::Result<StoredImage> {
    let (width, height) = dimensions(image)?;
    let (image_path, thumbnail_path) = paths_for(dir, hash);

    let result = write_original(&image_path, image, width, height)
        .and_then(|()| write_thumbnail(&thumbnail_path, image, width, height));
    if let Err(err) = result {
        remove_files(&[
            image_path.to_string_lossy().into_owned(),
            thumbnail_path.to_string_lossy().into_owned(),
        ]);
        return Err(err);
    }

    Ok(StoredImage {
        hash: hash.to_owned(),
        image_path,
        thumbnail_path,
        width,
        height,
    })
}

/// 读取落盘图片并解码为 RGBA8 像素（粘贴侧写回剪贴板用）。
pub fn load_rgba(path: &Path) -> Result<ImageData<'static>, ImageError> {
    let decoded = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let rgba = decoded.into_rgba8();
    Ok(ImageData {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        bytes: Cow::Owned(rgba.into_raw()),
    })
}

/// 尽力删除一组文件：文件不存在静默，其余失败只记日志（磁盘清理不阻断主流程）。
pub fn remove_files(paths: &[String]) {
    for path in paths {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => log::warn!("删除图片文件失败 {path}: {err}"),
        }
    }
}

/// 校验并取出图片尺寸：arboard 用 usize 表示，PNG 只接受 u32，且字节数必须等于 宽×高×4
fn dimensions(image: &ImageData<'_>) -> io::Result<(u32, u32)> {
    let width = u32::try_from(image.width)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "图片宽度超出范围"))?;
    let height = u32::try_from(image.height)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "图片高度超出范围"))?;
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    if expected != Some(image.bytes.len()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "图片像素字节数与宽高不匹配",
        ));
    }
    Ok((width, height))
}

/// 原图：无损 PNG，Fast 压缩换编码时延（大截图从秒级降到百毫秒级），体积略大可接受
fn write_original(path: &Path, image: &ImageData<'_>, width: u32, height: u32) -> io::Result<()> {
    let writer = BufWriter::new(File::create(path)?);
    PngEncoder::new_with_quality(writer, CompressionType::Fast, FilterType::Adaptive)
        .write_image(&image.bytes, width, height, ExtendedColorType::Rgba8)
        .map_err(io::Error::other)
}

/// 缩略图：长边缩到 [`THUMBNAIL_MAX_EDGE`] 以内、保持比例、只缩不放；默认压缩追求小体积
fn write_thumbnail(path: &Path, image: &ImageData<'_>, width: u32, height: u32) -> io::Result<()> {
    let (thumb_width, thumb_height) = thumbnail_size(width, height);
    // 借用像素构造视图而非 to_vec 复制：4K 截图的 RGBA 约 33MB，没必要为缩略图再拷一份。
    // 尺寸不变时 thumbnail 每个源像素恰好映射到一个目标像素，结果与原图一致，无需单独分支
    let source: ImageBuffer<Rgba<u8>, &[u8]> =
        ImageBuffer::from_raw(width, height, &image.bytes[..]).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "图片像素字节数与宽高不匹配")
        })?;
    image::imageops::thumbnail(&source, thumb_width, thumb_height)
        .save_with_format(path, ImageFormat::Png)
        .map_err(io::Error::other)
}

/// 计算缩略图尺寸：长边 ≤ [`THUMBNAIL_MAX_EDGE`]，短边按比例缩放且至少 1 像素；小图原样返回
fn thumbnail_size(width: u32, height: u32) -> (u32, u32) {
    let longest = width.max(height);
    if longest <= THUMBNAIL_MAX_EDGE {
        return (width, height);
    }
    let scale = THUMBNAIL_MAX_EDGE as f64 / longest as f64;
    let scaled = |edge: u32| ((edge as f64 * scale).round() as u32).max(1);
    (scaled(width), scaled(height))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每个测试独占一个临时目录，测完由守卫清理
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default();
            let dir = std::env::temp_dir().join(format!(
                "zach-tools-image-store-{}-{name}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).expect("创建临时目录失败");
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 构造一张确定内容的 RGBA 渐变图，保证像素非平凡便于验证往返
    fn sample_image(width: u32, height: u32) -> ImageData<'static> {
        let mut bytes = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                bytes.extend_from_slice(&[(x % 256) as u8, (y % 256) as u8, 128, 255]);
            }
        }
        ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(bytes),
        }
    }

    fn png_dimensions(path: &Path) -> (u32, u32) {
        ImageReader::open(path)
            .expect("打开图片失败")
            .into_dimensions()
            .expect("读取尺寸失败")
    }

    #[test]
    fn save_writes_original_and_thumbnail_and_shrinks_large_image() {
        let dir = TempDir::new("save-large");
        let image = sample_image(800, 400);
        let hash = hash_image(800, 400, &image.bytes);

        let stored = save(dir.path(), &image, &hash).expect("落盘失败");
        assert_eq!(stored.hash, hash);
        assert_eq!((stored.width, stored.height), (800, 400));
        assert!(stored.image_path.is_file());
        assert!(stored.thumbnail_path.is_file());
        assert_eq!(paths_for(dir.path(), &hash).0, stored.image_path);

        assert_eq!(png_dimensions(&stored.image_path), (800, 400));
        let (tw, th) = png_dimensions(&stored.thumbnail_path);
        assert_eq!(tw.max(th), THUMBNAIL_MAX_EDGE);
        assert_eq!((tw, th), (200, 100));
    }

    #[test]
    fn save_does_not_upscale_small_thumbnail() {
        let dir = TempDir::new("save-small");
        let image = sample_image(40, 30);
        let hash = hash_image(40, 30, &image.bytes);

        let stored = save(dir.path(), &image, &hash).expect("落盘失败");
        assert_eq!(png_dimensions(&stored.thumbnail_path), (40, 30));
    }

    #[test]
    fn load_rgba_roundtrips_with_same_hash() {
        let dir = TempDir::new("roundtrip");
        let image = sample_image(64, 48);
        let hash = hash_image(64, 48, &image.bytes);
        let stored = save(dir.path(), &image, &hash).expect("落盘失败");

        let loaded = load_rgba(&stored.image_path).expect("读取原图失败");
        assert_eq!((loaded.width, loaded.height), (64, 48));
        assert_eq!(
            hash_image(loaded.width as u32, loaded.height as u32, &loaded.bytes),
            hash
        );
    }

    #[test]
    fn hash_distinguishes_dimensions_with_same_bytes() {
        let bytes = vec![0u8; 2 * 4 * 4];
        assert_ne!(hash_image(2, 4, &bytes), hash_image(4, 2, &bytes));
    }

    #[test]
    fn save_rejects_mismatched_byte_length() {
        let dir = TempDir::new("mismatch");
        let image = ImageData {
            width: 4,
            height: 4,
            bytes: Cow::Owned(vec![0u8; 10]),
        };
        assert!(save(dir.path(), &image, "bad").is_err());
        let (image_path, thumbnail_path) = paths_for(dir.path(), "bad");
        assert!(!image_path.exists());
        assert!(!thumbnail_path.exists());
    }

    #[test]
    fn remove_files_ignores_missing_and_deletes_existing() {
        let dir = TempDir::new("remove");
        let existing = dir.path().join("exists.png");
        std::fs::write(&existing, b"png").expect("写文件失败");
        let missing = dir.path().join("missing.png");

        remove_files(&[
            existing.to_string_lossy().into_owned(),
            missing.to_string_lossy().into_owned(),
        ]);
        assert!(!existing.exists());
    }

    #[test]
    fn thumbnail_size_keeps_ratio_and_min_edge() {
        assert_eq!(thumbnail_size(100, 50), (100, 50));
        assert_eq!(thumbnail_size(400, 200), (200, 100));
        assert_eq!(thumbnail_size(200, 1000), (40, 200));
        assert_eq!(thumbnail_size(4000, 1), (200, 1));
    }
}
