# 执行计划:剪贴板图片链路

> 按 `design.md` 分层落地,每步可独立编译验证。Rust 命令均在 `src-tauri/` 下执行,前端命令在仓库根目录。

## 前置

- 编码前读 `.trellis/spec/tauri/index.md` 与 `.trellis/spec/frontend/index.md`,以及 `implement.jsonl` 列出的分层规范
- 提交规范见 `.trellis/spec/guides/project-conventions.md`

## 步骤

### Step 1 依赖与配置(R6)
- [x] `src-tauri/Cargo.toml`:`tauri` features 追加 `protocol-asset`;新增 `image = { version = "0.25", default-features = false, features = ["png"] }`
- [x] `src-tauri/tauri.conf.json5`:`app.security.assetProtocol = { enable: true, scope: ["$APPLOCALDATA/clipboard-images/**"] }`,补中文注释(含 CSP 收紧提示)
- [x] `db.rs`:数据根目录改为 `app_local_data_dir()`(D5 追加)
- 验证:`cargo check`

### Step 2 image_store 服务(R2)
- [x] 新建 `src-tauri/src/services/image_store.rs`,在 `services.rs` 注册
- [x] 实现 `hash_image` / `paths_for` / `save` / `load_rgba` / `remove_files`,常量 `IMAGE_DIR_NAME = "clipboard-images"`、`THUMBNAIL_MAX_EDGE = 200`
- [x] 单测(`tempfile` 或 `std::env::temp_dir()` + 随机子目录):save 后两文件存在、缩略图长边 ≤ 200 且小图不放大、`load_rgba` 往返 hash 一致、`remove_files` 对不存在文件静默
- 验证:`cargo test image_store`

### Step 3 存储层(R3)
- [x] `clipboard_store.rs`:新增 `ImageRow`、`insert_image`、`touch_by_hash`、`ClipContent`、`content`;`delete` 返回 `Option<Removed>`,`prune` 返回 `Removed`(`DELETE ... RETURNING`);删除 `text_content`
- [x] 补单测:insert_image + 重复 hash 只上浮;`content` 三态;`delete` / `prune` 返回图片路径且文本行不产生路径;更新现有测试中手插 image 行的用例
- 验证:`cargo test clipboard_store`

### Step 4 AppState 与启动(R2)
- [x] `state.rs`:`AppState::new(db, image_dir)` + `image_dir()` 访问器
- [x] `lib.rs` setup:`create_dir_all(app_local_data_dir/clipboard-images)` 后构造 AppState
- [x] `error.rs`:新增 `ImageFile(String)` 变体(中文文案)与 `From<image::ImageError>`
- 验证:`cargo check`

### Step 5 平台层(R1)
- [x] `platform.rs`:`ClipboardCapture` 改为 `Text` / `Image` 枚举;stub 不变
- [x] `win_monitor.rs`:`read_clipboard_text` → `read_capture`,文本优先、其次 `get_image`,重试策略保留;更新模块文档
- 验证:`cargo check`(Windows)

### Step 6 入库编排(R2、R3)
- [x] `clipboard_ingest.rs`:按 capture 分派;图片分支 `spawn_blocking` hash → 自写守卫 → `touch_by_hash` → `save` → `insert_image`(失败清文件)→ prune → 删文件 → emit;文本分支 prune 后同样删文件
- 验证:`cargo clippy`,手工:截图后列表出现条目、文件落盘;重复复制不新增文件

### Step 7 粘贴/复制(R4)
- [x] `paste.rs`:`copy_text_to_clipboard` 泛化为 `copy_to_clipboard(state, ClipContent)`,图片分支 `load_rgba` → hash → 标记 → `set_image`,失败 `clear_self_write`
- [x] `commands/clipboard.rs`:`paste_clipboard_item` / `copy_clipboard_item` 走 `store::content` + `copy_to_clipboard`;`delete_clipboard_item` 删文件;更新文档注释(去掉「仅支持文本」)
- 验证:手工 AC4 / AC5 / AC6 / AC10

### Step 8 前端(R5)
- [x] `lib/api.ts`:新增 `toAssetUrl`;修正 `kind` 注释
- [x] `lib/tabs.ts`:追加图片 tab
- [x] `ClipboardListItem.vue`:缩略图渲染 + 「图片 · W×H」预览文案
- [x] `ClipboardDetailPane.vue`:meta 按 kind 分支;图片正文渲染
- [x] 若有 `toAssetUrl` 纯函数逻辑可测(非 Tauri 返回空串),补 vitest
- 验证:`bun run lint && bun run test && bun run build`;手工 AC1 / AC8 / AC9

### Step 9 全量门禁与回归
- [x] `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [x] `bun run format && bun run lint && bun run test && bun run build`
- [ ] 手工回归 AC12(文本采集/粘贴/复制/删除/搜索)与 AC2 / AC3 / AC7
- [x] 派发 `trellis-check` 做规范符合性检查

## 风险与回滚点

| 风险 | 缓解 | 回滚点 |
|------|------|--------|
| `protocol-asset` 未生效导致 `<img>` 空白 | Step 1 后先用 devtools 验证任意 `asset://` URL 可访问 | Step 1 |
| 自写 hash 与采集 hash 不一致产生重复条目 | hash 函数唯一定义在 `image_store::hash_image`,两侧共用;Step 2 单测覆盖 PNG 往返 | Step 7 |
| 大图编码耗时阻塞 | `spawn_blocking` + PNG Fast;超大图无上限是已接受的决策 | — |
| `DELETE ... RETURNING` 在旧 SQLite 不可用 | sqlx 内置 libsqlite3 ≥ 3.35,`cargo test` 即可验证 | Step 3 |
| 修改 `ClipboardCapture` 破坏 stub 编译 | stub 仅用类型名,不解构;Step 5 `cargo check` | Step 5 |

## 提交拆分建议

1. `chore(deps): 引入 image 与 tauri protocol-asset,开启剪贴板图片资源协议`
2. `feat(clipboard): 图片落盘与存储层支持 image 条目`(Step 2~4)
3. `feat(clipboard): Windows 监听捕获图片并入库`(Step 5~6)
4. `feat(clipboard): 图片条目粘贴/复制与删除清理文件`(Step 7)
5. `feat(clipboard): 前端渲染图片缩略图、详情预览与图片 tab`(Step 8)
