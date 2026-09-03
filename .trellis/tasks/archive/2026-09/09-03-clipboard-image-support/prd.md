# 剪贴板支持图片复制与粘贴

## Goal

剪贴板历史打通图片链路:Windows 监听捕获图片 → 原图与缩略图落盘去重 → 存储层写入 `image` 条目 → 粘贴/复制回写系统剪贴板 → 前端列表缩略图与详情预览。用户复制的截图/图片能像文本一样出现在历史里,选中后一键粘贴回原应用。

## Background(代码现状)

- 表结构已为图片预留字段:`image_path` / `thumbnail_path` / `image_width` / `image_height`,`kind` CHECK 含 `image`,`content_hash` 注释声明 image 取像素数据 hash(`src-tauri/migrations/0001_create_clipboard_items.sql`)
- `ClipboardKind::Image` 与 `ClipboardItem` 图片字段在 Rust / TS 两侧均已镜像(`clipboard_store.rs:26-51`,`src/tools/clipboard/lib/api.ts:12-25`);`list` 的 `kind` 过滤已可用并有单测
- 监听线程只读文本:`read_clipboard_text` 遇 `ContentNotAvailable` 直接丢弃(`win_monitor.rs:100-114`);通道载荷 `ClipboardCapture { text }` 是单一文本结构(`platform.rs:5-8`)
- 入库循环只有文本路径:`ingest` → `hash_text` → `upsert_text`(`clipboard_ingest.rs:28-46`)
- 自写回环标记基于 hash 字符串(`state.rs:35-58`),与内容类型无关;arboard 3.6.1 写图片同时放 PNG + CF_DIBV5、读图优先 PNG(`arboard/src/platform/windows.rs:624-647, 712-726`),像素往返无损,图片可复用该守卫
- 粘贴/复制只有 `copy_text_to_clipboard`,非文本条目报 `UnsupportedKind`(`paste.rs`,`commands/clipboard.rs:87-92`)
- `delete` / `prune` 仅删 SQL 行,不感知磁盘文件(`clipboard_store.rs:226-240`)
- 前端列表项对 image 仅渲染 lucide 图标占位,注释明确「渲染真缩略图是二期工作」(`ClipboardListItem.vue:14`);详情栏只渲染文本(`ClipboardDetailPane.vue`);类型 tab 注册表注释预留了 `{ key: "image", label: "图片", kind: "image" }`(`lib/tabs.ts`)
- `image` crate 0.25(png feature)已是 arboard 在 Windows 下的依赖,显式引入零额外编译成本
- `tauri.conf.json5` 未开启 `assetProtocol`、`tauri` crate 未开 `protocol-asset` feature;`csp: null`
- 数据目录:`app.path().app_data_dir()`,SQLite 已落在此处(`db.rs:12-16`)
- 竞品参考见 `research-utools-super-clipboard.md`

## Decisions(已拍板)

- **D1 容量**:图片与文本共用 `MAX_HISTORY_ITEMS = 500` 总容量,不设图片独立配额;**不设单图像素/体积上限**(用户明确选择);原图 PNG 无损落盘;「按天过期」不在本期
- **D2 列表呈现**:保持 44px 统一行高,缩略图填进现有 28×28 类型图标格(`object-cover`),中部文字显示「图片 · 宽×高」;大图由右侧详情栏承担
- **D3 多图语义**:系统剪贴板同一时刻只有一幅位图;资源管理器多选图片文件是 CF_HDROP 文件列表,属 `files` 类型(uTools 亦按文件处理)。本期 `image` 只处理单幅位图,文件列表沿现状忽略
- **D4 文本/图片并存**:有非空文本记文本,不再读图片;仅当剪贴板无文本时才读图(与 uTools 读取顺序一致)
- **D5 数据根目录**(实施中追加):所有持久化数据统一放 `app_local_data_dir()`(`%LOCALAPPDATA%\<identifier>`),SQLite 库与 `clipboard-images/` 同根,并与已在此处的 `logs/`、WebView2 `EBWebView/` 合并成一个顶层目录,方便整体清理;放弃原先的 `app_data_dir()`(Roaming)。当前没有用户,不做老库迁移

## Requirements

### R1 监听捕获图片(Windows)
- 监听线程按「文本 → 图片」顺序读取:文本非空则发文本;文本 `ContentNotAvailable` 时尝试 `get_image`;两者皆无则忽略
- 图片读取沿用现有被占用重试策略(3 次、线性退避)
- 非 Windows 平台 stub 不变

### R2 落盘与去重
- 图片存于 `app_local_data_dir()/clipboard-images/`(与 SQLite 库同根,见 D5),原图 `<hash>.png`、缩略图 `<hash>.thumb.png`;目录在应用启动时确保存在
- `content_hash = blake3(宽 ‖ 高 ‖ RGBA 像素)`,全局唯一键;重复复制同一图片只刷新 `last_used_at` 上浮,**不重复编码、不重复写文件**
- 缩略图:长边 ≤ 200px、保持比例、只缩不放
- 编码、hash、缩略图生成均不阻塞 tokio 运行时(`spawn_blocking`),也不阻塞监听消息循环
- 写文件失败不入库;入库失败清理已写文件,避免孤儿

### R3 存储层
- 新增图片 upsert:插入 `kind='image'` 行(路径、宽高、hash),hash 冲突时只更新 `last_used_at`;返回预览形态条目
- `delete` / `prune` 返回被删行的图片文件路径,服务层据此删除磁盘文件(尽力而为,失败只记日志)
- 收藏图片不参与容量清理(现有语义不变)

### R4 粘贴与复制
- `paste_clipboard_item` / `copy_clipboard_item` 支持 image 条目:读原图 PNG → 解码 RGBA → 计算 hash 打自写标记 → `set_image` 写剪贴板;后续隐藏面板、焦点还原、Ctrl+V、`touch` 流程与文本一致
- 原图文件缺失时报错(文案面向用户),不崩溃
- 自写图片触发的监听事件被 hash 守卫吞掉,不产生重复条目

### R5 前端
- 前端通过 Tauri asset 协议加载本地图片(`convertFileSrc`),配置 `assetProtocol.enable` + scope 仅限 `$APPLOCALDATA/clipboard-images/**`
- 列表项:image 条目在图标格渲染缩略图,文字「图片 · W×H」,时间与收藏星标同文本行
- 详情栏:头部元信息「图片 · W×H · 相对时间」,操作按钮(收藏/复制/删除)同文本;正文居中等比显示原图(`object-contain`,不超出面板)
- 类型 tab 增加「图片」;关键字搜索仍只匹配文本(image 条目在有搜索词时自然不出现)
- 非 Tauri 运行时(浏览器预览)图片 URL 转换降级为空串,不报错

### R6 配置与依赖
- `Cargo.toml`:`tauri` 增加 `protocol-asset` feature;新增 `image = { version = "0.25", default-features = false, features = ["png"] }`
- `tauri.conf.json5`:`app.security.assetProtocol = { enable: true, scope: ["$APPLOCALDATA/clipboard-images/**"] }`,并在注释中记录将来收紧 CSP 时需放行 `img-src asset: http://asset.localhost`
- `db.rs` / `lib.rs`:数据根目录由 `app_data_dir()` 改为 `app_local_data_dir()`(D5)
- 无需新增 capability 权限(asset 协议由 config scope 管控)

## Acceptance Criteria

- [ ] AC1 截图工具截图(剪贴板仅位图)后,历史列表顶部秒级出现 image 条目,图标格显示缩略图,文字为「图片 · W×H」
- [x] AC2 再次复制同一张图,不新增条目,原条目上浮到顶部;`clipboard-images/` 目录文件数不变(已用 PowerShell `Clipboard.SetImage` 复现验证:仅触发 `touch_by_hash`,文件数保持 2)
- [x] AC3 Excel 复制单元格(文本 + 位图并存)只产生文本条目(已用 `DataObject` 同时放文本与位图验证:仅 INSERT text 行,无新文件)
- [ ] AC4 选中 image 条目按 Enter:面板收起,原应用(如画图/微信)收到粘贴的图片,像素尺寸与原图一致;历史中不出现新的重复条目
- [ ] AC5 点击「仅复制」后在其他应用 Ctrl+V 得到该图片,面板保持打开,按钮短暂反馈「已复制」
- [ ] AC6 删除 image 条目后,对应 `<hash>.png` 与 `<hash>.thumb.png` 从磁盘消失
- [ ] AC7 历史超过 500 条触发清理时,被清理的 image 行对应文件同步删除;收藏的图片不被清理
- [ ] AC8 详情栏显示完整原图,等比缩放不溢出面板,头部元信息正确
- [ ] AC9 「图片」tab 只列 image 条目;输入搜索词后 image 条目不出现;切回「全部」恢复
- [ ] AC10 原图文件被手动删除后粘贴该条目,前端收到可读中文错误,应用不崩溃
- [x] AC11 `cargo fmt --check` / `cargo clippy` 无警告 / `cargo test` 通过(含图片 upsert、delete/prune 返回路径的单测);`bun run lint` / `bun run test` / `bun run build` 通过(18 个 Rust 单测 + 14 个 vitest 用例)
- [ ] AC12 文本条目的采集、粘贴、复制、删除、搜索行为与改动前一致(回归)

## Out of Scope

- `files` 类型(复制文件/文件夹路径列表)入库与多图文件展开
- 非 Windows 平台的图片监听与写回(沿用 stub)
- 按天自动过期、图片独立配额、单图体积上限
- 图片编辑(旋转/裁剪)、OCR、以图搜图、另存为
- 启动时孤儿文件扫描(有行无文件 / 有文件无行)——写入顺序已尽量避免,极端崩溃场景遗留少量孤儿可接受
- 列表行加高 / 大缩略图布局(D2 选定 A 方案,后续可单独调整)
