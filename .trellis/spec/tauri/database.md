# 数据库

> SQLite(WAL)+ sqlx,存储层是纯函数集合:只收 `&SqlitePool`,不依赖 tauri,可用内存库单测。

---

## 初始化与生命周期

- 连接池在 setup 里初始化(`db::init_pool`):库文件放 `app_data_dir()`,`create_if_missing` + WAL 模式,池上限 4 连接(`src-tauri/src/db.rs`)
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
- 写操作返回**是否确有其事**:`UPDATE` / `DELETE` 后用 `rows_affected() > 0` 返回 `bool`,由命令层决定是否映射成 `ItemNotFound`(`set_favorite` / `delete`)
- 双层语义用嵌套 Option 表达并写注释:`text_content` 返回 `Option<Option<String>>`——外层「条目存在吗」,内层「是文本条目吗」
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

历史类数据入库后立即清理:保留最新 N 条,受保护标记(收藏)不清理(`prune` + `MAX_HISTORY_ITEMS`);超大单条在入库前跳过(`MAX_TEXT_BYTES` 上限判断在 `clipboard_ingest.rs`)。

---

## 存储层测试(必写)

存储层函数必须带文件内单测,这是后端目前唯一的成建制测试层,新增查询不写测试不算完成。既定手法(`clipboard_store.rs` 的 `mod tests`):

- `:memory:` 连接池 + 跑真实 migration,不 mock
- 辅助函数直接 UPDATE 时间戳,构造确定的排序场景(`set_last_used`)
- 用例覆盖不变量而非实现:去重上浮、清理保收藏、LIKE 转义、翻页遇删除不跳行、同毫秒决胜、预览截断与原文完整性
- 断言失败信息用中文 `expect("入库失败")`

运行:`cargo test`(在 `src-tauri/` 下)。
