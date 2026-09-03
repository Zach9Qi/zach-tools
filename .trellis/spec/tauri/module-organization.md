# 模块组织

> 入口只做装配,业务分层放置。判断代码放哪:先问「它是 IPC 入口、业务编排、纯存储,还是平台 API」。

---

## 真实结构

```
src-tauri/src/
├── main.rs              # 仅入口:调 zach_tools_lib::run()(含 Windows 隐藏控制台的 cfg_attr,勿删)
├── lib.rs               # Builder 装配:plugin、db 初始化、manage、generate_handler、setup、RunEvent
├── commands.rs          # 模块根:pub mod clipboard; pub mod launcher;
├── commands/            # 命令层,按领域一个文件
├── services.rs          # 模块根:声明五个服务
├── services/
│   ├── clipboard_store.rs   # 纯存储:只依赖 sqlx,不依赖 tauri,带完整单测
│   ├── clipboard_ingest.rs  # 采集链路编排:平台线程 → 通道 → 入库循环
│   ├── launcher_window.rs   # 窗口开合编排(show/hide/toggle/失焦/托盘联动)
│   ├── paste.rs             # 粘贴编排:写剪贴板 → 还原焦点 → 注入 Ctrl+V
│   └── tray.rs              # 系统托盘
├── platform.rs          # 平台层统一入口(facade):类型定义 + cfg 分发 + pub use
├── platform/            # win_input / win_monitor / win_window + stub
├── db.rs                # 连接池初始化 + migration
├── state.rs             # AppState 定义
├── error.rs             # AppError 统一错误
└── migrations/          # sqlx 迁移(编号 SQL 文件)
```

---

## 硬性规则

- **目录模块用 2018 写法**:`foo.rs` + `foo/` 子目录,**不新增 `mod.rs`**(编辑器标签能看出模块名);`commands.rs` / `services.rs` / `platform.rs` 都是这个形态
- `main.rs` 只做入口;`lib.rs` 只做 Builder 装配(plugin 注册、`manage`、`generate_handler!`、setup、RunEvent 处理),桌面端专属装配拆成 `#[cfg(desktop)] fn setup_desktop`
- 每个模块文件开头写 `//!` 模块级注释,一句话说清职责与数据流向(现有全部模块如此,如 `clipboard_ingest.rs` 开头的链路图)

---

## 服务层的两种形态

服务层内部有意区分两类,新增服务时先归类:

1. **纯逻辑服务**(`clipboard_store.rs`):不依赖任何 tauri 类型,函数签名只收 `&SqlitePool` 与普通参数,错误返回底层错误类型(`sqlx::Error`)由上层转换。可直接用内存库单测,文件内带 `#[cfg(test)] mod tests`
2. **编排服务**(`launcher_window.rs`、`paste.rs`、`clipboard_ingest.rs`、`tray.rs`):需要操作窗口 / 事件 / 状态,依赖 tauri 类型,但一律用泛型 `R: Runtime` 收参(`AppHandle<R>` / `WebviewWindow<R>`),不写死具体 runtime

能下沉到纯逻辑的部分尽量下沉:`paste.rs` 的剪贴板写入是编排,但 hash 计算在 `clipboard_store::hash_text`。

---

## 常量归属

- 模块内约定值定义在使用它的模块顶部,`pub` 与否按需要:窗口 label 与事件名在 `launcher_window.rs`,分页上限在 `commands/clipboard.rs`,容量与截断上限在 `clipboard_store.rs`(供 ingest 引用,`pub`)
- 与前端共享的值(窗口宽度、事件名、默认分页大小)无法编译期共享,靠**两侧注释互指**保持同步:窗口宽度以 `tauri.conf.json5` 的 `width` 为唯一定义处（前端 `src/lib/window.ts` 动态读 `window.innerWidth`，不再另存副本）；高度上限由前端 CSS（`src/launcher/components/LauncherPanel.vue` 的 `max-h-150`）封顶。改一侧必须检查另一侧

---

## 常见错误

- 往 `lib.rs` 里堆业务逻辑——只允许装配代码,业务进 services
- 新服务直接依赖具体 runtime 类型——用 `R: Runtime` 泛型
- 平台 API 散落在服务层——一律收进 `platform/`,经 `platform.rs` 的 facade 导出
