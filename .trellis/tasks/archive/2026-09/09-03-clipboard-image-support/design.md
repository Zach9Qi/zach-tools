# 技术设计:剪贴板图片链路

> 对应 `prd.md` R1~R6。遵循 `.trellis/spec/tauri/` 分层:commands 薄、services 编排/纯存储、platform 隔离 Win32。

## 1. 架构与边界

```
platform/win_monitor.rs   读剪贴板:文本优先,其次图片 → ClipboardCapture::{Text, Image}
        │  mpsc (unbounded)
services/clipboard_ingest.rs   分派:Text → 现有路径;Image → spawn_blocking(hash/查重/编码/缩略图) → 入库 → prune → emit
services/image_store.rs  (新)  纯 fs + image crate:hash、PNG 编码、缩略图、写/删文件;不依赖 tauri,可单测
services/clipboard_store.rs    新增 upsert_image / touch_by_hash / content(id);delete & prune 返回被删图片路径
services/paste.rs              copy_to_clipboard(ClipContent) 统一文本/图片写回 + 自写标记
commands/clipboard.rs          paste/copy 改为按 ClipContent 分派;delete 后清理文件
state.rs                       AppState 新增 image_dir: PathBuf
前端 lib/api.ts + 组件          convertFileSrc 包装、缩略图/详情渲染、图片 tab
```

## 2. 数据契约

### 2.1 通道载荷(`platform.rs`)

```rust
pub enum ClipboardCapture {
    Text(String),
    /// arboard 读出的 RGBA8 像素(已持有,'static)
    Image(arboard::ImageData<'static>),
}
```

stub 平台 `spawn_monitor` 签名不变。

### 2.2 图片元数据(`image_store.rs`)

```rust
pub struct StoredImage {
    pub hash: String,          // blake3 hex
    pub image_path: PathBuf,   // <dir>/<hash>.png
    pub thumbnail_path: PathBuf, // <dir>/<hash>.thumb.png
    pub width: u32,
    pub height: u32,
}
```

- `hash_image(w, h, rgba) -> String`:`blake3` 更新顺序 `w.to_le_bytes() ‖ h.to_le_bytes() ‖ rgba`。粘贴侧解码 PNG 后用同一函数,保证与采集侧一致
- `paths_for(dir, hash) -> (image_path, thumbnail_path)`:文件名规则的唯一定义处
- `save(dir, image: &ImageData, hash) -> io::Result<StoredImage>`:
  - 原图:`PngEncoder::new_with_quality(CompressionType::Fast, FilterType::Adaptive)`,快速压缩换编码时延(4K 截图从秒级降到百毫秒级)
  - 缩略图:`image::imageops::thumbnail` 缩到长边 ≤ `THUMBNAIL_MAX_EDGE = 200`,只缩不放;默认 PNG 压缩(体积小、文件少)
  - 先写临时名再 rename?——单机本地写,采用「原图 → 缩略图」顺序直接写;任一失败调用 `remove(dir, hash)` 清理后返回 Err
- `load_rgba(path) -> Result<ImageData<'static>, ImageError>`:粘贴侧解码
- `remove_files(paths: &[String])`:逐个 `remove_file`,`NotFound` 静默,其余 `log::warn!`

### 2.3 存储层(`clipboard_store.rs`)

```rust
pub struct ImageRow<'a> { pub hash: &'a str, pub image_path: &'a str, pub thumbnail_path: &'a str, pub width: i64, pub height: i64 }

pub async fn touch_by_hash(pool, hash) -> Result<Option<ClipboardItem>>   // 命中则刷新 last_used_at 并返回预览
pub async fn insert_image(pool, row: ImageRow<'_>) -> Result<ClipboardItem>
pub enum ClipContent { Text(String), Image { path: String } }
pub async fn content(pool, id) -> Result<Option<Option<ClipContent>>>      // 替代 text_content:外层条目是否存在,内层是否受支持类型
pub struct Removed { pub image_files: Vec<String> }                      // 被删行涉及的图片/缩略图路径
pub async fn delete(pool, id) -> Result<Option<Removed>>                 // None = 不存在
pub async fn prune(pool, keep) -> Result<Removed>
```

- `delete` / `prune` 用 `DELETE ... RETURNING image_path, thumbnail_path` 一条 SQL 拿回路径(SQLite ≥ 3.35 支持;sqlx 内置 SQLite 满足)
- 图片入库拆成「先 `touch_by_hash` 查重,命中即返回」+「未命中才编码写文件再 `insert_image`」,避免重复复制时白编码一次;`insert_image` 仍带 `ON CONFLICT(content_hash) DO UPDATE SET last_used_at` 兜底并发竞态

### 2.4 跨端结构

`ClipboardItem` 字段不变;image 条目 `imagePath` / `thumbnailPath` 为绝对路径字符串,`imageWidth` / `imageHeight` 为像素值,`textPreview` / `textLength` 为 null。`api.ts` 只改注释(去掉「image / files 为预留」)。

## 3. 数据流

### 3.1 采集(R1、R2、R3)

```
WM_CLIPBOARDUPDATE
 → read_capture():
     get_text  Ok(非空) → Text
               Err(ContentNotAvailable) → get_image  Ok → Image / Err(ContentNotAvailable) → None
               其他 Err → 退避重试(沿用 READ_RETRIES)
 → tx.send
ingest(Image(img)):
   spawn_blocking { hash = hash_image(...) }                  // 大图 hash 不占 tokio 线程
   if state.take_self_write_if_matches(&hash) → return
   if let Some(item) = store::touch_by_hash(db, &hash) → emit(item); return   // 重复图:零编码
   stored = spawn_blocking { image_store::save(&state.image_dir, &img, &hash) }?
   item = store::insert_image(db, row).await
          .inspect_err(|_| image_store::remove_files(stored paths))?  // 入库失败清文件
   removed = store::prune(db, MAX_HISTORY_ITEMS).await?; image_store::remove_files(&removed.image_files)
   emit(EVENT_NEW_ITEM, item)
```

文本分支逻辑原样保留(含 `MAX_TEXT_BYTES` 检查),只是 prune 之后多一步删文件(文本清理也可能淘汰图片行)。

### 3.2 粘贴/复制(R4)

```
commands::paste_clipboard_item(id):
   content = store::content(db, id) → ItemNotFound / UnsupportedKind(files)
   paste::copy_to_clipboard(state, content):
       Text(t)        → 现有逻辑
       Image{path}    → spawn_blocking { img = image_store::load_rgba(path)?; hash = hash_image(img) }
                        state.mark_self_write(hash); spawn_blocking { set_image(img) } 失败则 clear_self_write
   launcher_window::hide; paste::deliver_paste; store::touch
```

`load_rgba` 失败(文件丢失/损坏)→ 新增 `AppError::ImageFile(#[from] image::ImageError)`「图片文件读取失败: …」,命令返回给前端展示(实现时选 `#[from]` 而非 `String`,与兄弟变体写法一致,`?` 直接传播)。

### 3.3 删除(R3)

`commands::delete_clipboard_item`:`store::delete` → `Some(removed)` 则 `image_store::remove_files`;`None` → `ItemNotFound`。

### 3.4 前端(R5)

- `lib/api.ts` 新增 `toAssetUrl(path: string | null): string`:`isTauriRuntime()` 且非空 → `convertFileSrc(path)`,否则 `""`
- `ClipboardListItem.vue`:`kind === "image"` 时图标格内 `<img :src="toAssetUrl(item.thumbnailPath)" class="size-7 rounded-md object-cover">`,`preview` 计算改为 `图片 · ${w}×${h}`;`alt` 空(装饰性)
- `ClipboardDetailPane.vue`:`meta` 按 kind 分支;正文 `kind === "image"` 渲染居中容器 + `<img class="max-h-full max-w-full object-contain" :src="toAssetUrl(item.imagePath)">`,文本分支不变
- `lib/tabs.ts`:`KIND_TABS` 追加 `{ key: "image", label: "图片", kind: "image" }`(Tab 键轮切自动生效)

## 4. 配置

- `Cargo.toml`:`tauri` features 加 `protocol-asset`;新增 `image` 依赖(仅 png)
- `tauri.conf.json5` `app.security`:
  ```json5
  "assetProtocol": { "enable": true, "scope": ["$APPLOCALDATA/clipboard-images/**"] }
  ```
  注释记录:收紧 CSP 时需 `img-src 'self' asset: http://asset.localhost`
- `db.rs`:库文件根目录 `app_data_dir()` → `app_local_data_dir()`(PRD D5,所有持久化数据同根;`$APPLOCALDATA` 是 Tauri 内置 scope 变量,对应 `app_local_data_dir()`)
- `lib.rs` setup:`app_local_data_dir()/clipboard-images` `create_dir_all` 后传入 `AppState::new(pool, image_dir)`

## 5. 兼容与迁移

- 无 schema 变更,无 migration;老库中不存在 image 行,新老版本可互相打开
- 前端事件 `clipboard-new-item` 载荷结构不变,消费方按 id 去重逻辑照旧
- `text_content` 被 `content` 替代,仅内部调用方(commands)受影响

## 6. 取舍

| 决策 | 选择 | 备选 | 原因 |
|------|------|------|------|
| 前端取图 | asset 协议 + `convertFileSrc` | 命令返回 base64 | 缩略图 20~60KB × 50/页,base64 走 IPC JSON 太重;asset 协议按需流式读盘 |
| 编码位置 | ingest 内 `spawn_blocking` | 监听线程内编码 | 监听线程是 Win32 消息循环,阻塞会积压后续剪贴板事件 |
| 查重时机 | 先 `touch_by_hash` 再编码 | 直接 upsert | 重复复制是高频场景,避免白编码大图 |
| 原图压缩 | PNG Fast | PNG Default / JPEG | Fast 编码快数倍、体积略大;无损是 D1 要求 |
| 文件清理 | SQL `RETURNING` 路径 → 服务层删 | 存储层直接删文件 | 存储层保持「不依赖 fs、可内存库单测」 |
| 孤儿治理 | 写入顺序保证 + 不做扫描 | 启动时全目录比对 | MVP 收敛,PRD 已列 Out of Scope |

## 7. 回滚

- 全部改动在应用层,无数据迁移;回滚代码即可,已落盘的 `clipboard-images/` 与 image 行对旧版本只是「永不命中」的数据,不影响文本功能
- `assetProtocol` 与 `protocol-asset` feature 随代码一起回退
