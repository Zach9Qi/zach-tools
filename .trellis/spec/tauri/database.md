# 数据库

> SQLite(WAL)+ sqlx,存储层是纯函数集合:只收 `&SqlitePool`,不依赖 tauri,可用内存库单测。

---

## 初始化与生命周期

- 连接池在 setup 里初始化(`db::init_pool`):库文件放 `app_local_data_dir()`,`create_if_missing` + WAL 模式,池上限 4 连接(`src-tauri/src/db.rs`)
- **所有持久化数据统一放 `app_local_data_dir()`**(Windows 为 `%LOCALAPPDATA%\<identifier>`):SQLite 库、`clipboard-images/`、`logs/`(tauri-plugin-log 默认)与 WebView2 的 `EBWebView/` 同根,清理时删一个目录即可。**不用 `app_data_dir()`(Roaming)**——剪贴板历史是机器本地、体积大、可能含敏感内容的数据,不该跟随账号漫游;新增落盘数据沿用这个根
- migration 内嵌:`sqlx::migrate!("./migrations")` 在建池后立即执行,应用启动即保证 schema 最新
- 退出时显式 `pool.close()`(见[状态与并发](./state-and-concurrency.md)的优雅退出)

## migration 约定

- 文件放 `src-tauri/migrations/`,命名 `NNNN_动作_对象.sql`(`0001_create_clipboard_items.sql`)
- **只增不改**:已发布的 migration 不回头编辑,schema 变更加新编号文件
- 每列写 SQL 注释说明含义与单位;索引要注释「为什么需要这个序」(参照 0001 里对 `(last_used_at DESC, id DESC)` 索引服务于 keyset 分页的注释)
- 多形态数据共用一张表时,类型专属列可空 + `CHECK` 约束枚举合法值(`kind IN ('text','image','files')`)

---

## 存储层写法(clipboard_store.rs 为基准)

- 函数签名:`pub async fn xxx(pool: &SqlitePool, ...) -> Result<T, sqlx::Error>`,错误由命令层经 `#[from]` 转成 `AppError`,存储层不认识 `AppError`
- 写操作返回**是否确有其事**:`UPDATE` 后用 `rows_affected() > 0` 返回 `bool`(`set_favorite`),由命令层决定是否映射成 `ItemNotFound`
- 删除类操作要把**波及的磁盘文件交出去**:`delete` 返回 `Option<Removed>`(None = 不存在)、`prune` 返回 `Removed`,用一条 `DELETE ... RETURNING image_path, thumbnail_path` 取回被删 image 行的文件路径,由服务层(`clipboard_ingest` / 命令层)调 `image_store::remove_files` 清理。存储层自己**不碰 fs**,内存库单测才跑得起来
- 双层语义用嵌套 Option 表达并写注释:`content` 返回 `Option<Option<ClipContent>>`——外层「条目存在吗」,内层「类型能写回剪贴板吗」(`Text` 原文 / `Image { path }`,files 为 None)
- 同一内容多次入库走 hash 去重:文本 `upsert_text` 一条 `ON CONFLICT DO UPDATE`;图片因为编码写文件成本高,拆成先 `touch_by_hash`(命中即上浮返回)再 `insert_image`(仍带 `ON CONFLICT` 兜底并发竞态)
- 枚举列用 `#[derive(sqlx::Type)] + #[sqlx(rename_all = "lowercase")]`,JSON 列用 `sqlx::types::Json<T>`(`file_paths`)
- 动态条件用 `QueryBuilder` 逐段拼 + `push_bind`,**禁止**字符串格式化拼 SQL;共用的投影段提成常量 + 构造函数(`preview_query()`)
- 静态语句用 `sqlx::query` / `query_scalar` + `?1` 占位符

## LIKE 与用户输入

用户关键字进 LIKE 前必须转义通配符,配合 `ESCAPE '\'`(`escape_like`:反斜杠、`%`、`_` 三个字符);有对应测试用例锁行为(`list_escapes_like_wildcards`)。

---

## keyset 分页(列表查询的默认方案)

不用 offset 分页。要点(实现与测试见 `clipboard_store.rs`):

1. 排序键必须是**确定全序**:`(last_used_at DESC, id DESC)`,id 决胜同毫秒并列
2. 建配套复合索引(migration 0001)
3. 游标是值锚点:上一页末行的 `(last_used_at, id)`,下一页查 `(last_used_at, id) < (锚点)`——期间的插入 / 删除 / 置顶不会跳行或重行
4. 游标结构体跨端传输(`ListCursor` ↔ 前端 `ClipboardListCursor`)

## 容量治理

历史类数据入库后立即清理:保留最新 N 条,受保护标记(收藏)不清理(`prune` + `MAX_HISTORY_ITEMS`);超大单条在入库前跳过(`MAX_TEXT_BYTES` 上限判断在 `clipboard_ingest.rs`)。图片与文本共用同一容量,`prune` 返回的图片路径由 `clipboard_ingest` 统一删文件——文本入库把最旧的 image 行挤出容量时,文件也在这一步清掉。

---

## 存储层测试(必写)

存储层函数必须带文件内单测,这是后端目前唯一的成建制测试层,新增查询不写测试不算完成。既定手法(`clipboard_store.rs` 的 `mod tests`):

- `:memory:` 连接池 + 跑真实 migration,不 mock
- 辅助函数直接 UPDATE 时间戳,构造确定的排序场景(`set_last_used`);图片行用 `insert_sample_image` 夹具(固定尺寸 + `images/<hash>[.thumb].png` 路径),不手写 INSERT
- 用例覆盖不变量而非实现:去重上浮(文本 / 图片)、清理保收藏、LIKE 转义、翻页遇删除不跳行、同毫秒决胜、预览截断与原文完整性、`content` 三态、`delete` / `prune` 返回图片路径且文本行不产生路径
- 断言失败信息用中文 `expect("入库失败")`

运行:`cargo test`(在 `src-tauri/` 下)。
