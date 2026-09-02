# 质量与安全

> 后端改动的验收基线与安全配置约定。安全立场:前端视为不可信输入源,权限最小化。

---

## 校验命令(在 `src-tauri/` 下执行,提交前全过)

```bash
cargo fmt          # rustfmt 默认格式
cargo clippy       # 无警告
cargo test         # 存储层等单测
```

联调用仓库根的 `bun run tauri dev`。

CI(`.github/workflows/ci.yml`)在 **windows-latest 与 ubuntu-24.04** 双平台跑 `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test`:Windows 覆盖真实 Win32 平台层,Ubuntu 覆盖 `stub.rs`,任何 clippy 警告或跨平台编译破坏都会挡下 main 的 push / PR。本地把 clippy 警告清零即等价于过 CI。

---

## unwrap / expect 政策

- **业务代码禁止 `unwrap()`**;例外只有两处:入口装配的 `expect`(`lib.rs` 的 `.expect("启动应用失败")`)与测试代码的 `expect("中文失败说明")`
- 明确可忽略的 `Result` 用 `let _ =` 显式丢弃,仅限「失败无后果且无法处置」的调用,如窗口 show / hide / set_position(`launcher_window.rs`)
- 拿不到就没法继续、但不算错误的场景用 `let-else` 早返回(`launcher_window.rs` 的 `main_window` 查询链)
- 锁不 `unwrap`,走 `lock_or_recover`(见[状态与并发](./state-and-concurrency.md))

## 错误与日志

- 错误定义集中在 `error.rs`:thiserror 枚举 + `#[from]` 自动转换 + 手写 `Serialize` 输出 `to_string()`;**错误文案面向前端用户展示,写中文**,新增变体保持口径(`#[error("剪贴板访问失败: {0}")]`)
- 内部传播一律 `?`;「记日志继续」只用于常驻循环 / 尽力而为路径(ingest 循环、按键注入不足量)
- 日志用 `log` 宏(经 tauri-plugin-log 输出),**禁止 `println!` 调试**;级别:错误影响功能用 `error`,降级可继续用 `warn`,链路里程碑用 `info`,重试细节用 `debug`。debug 构建 Debug 级、发布 Info 级已在 `lib.rs` 配好

---

## 注释纪律

- 模块开头 `//!` 说明职责与数据流;导出函数、结构体、字段写 `///` 中文文档注释
- 跨端结构体逐字段注释是硬要求(见[命令与 IPC](./commands-and-ipc.md))
- 解释「为什么」的行注释写在决策点上,基准参照:`launcher_window.rs` 对「托盘悬停时跳过失焦收起」的因果链注释、`state.rs` 对锁中毒恢复安全性的说明

---

## 权限与安全配置

**capability 最小权限**(Tauri 2 安全模型,现状即范例):`capabilities/default.json` 只授 `core:default`、`core:window:allow-set-size`、`opener:default` 三项——`allow-set-size` 是为前端自适应窗口高度单独加的,这就是「用到才加、按窗口授权」的粒度。

- 新增 plugin 三步走:`Cargo.toml` 加 crate → `lib.rs` `.plugin(...)` 注册 → `capabilities/` 声明其权限;三步缺一前端调用即失败
- 新用 core 能力(窗口操作等)同样先查权限标识再加进 capability,不图省事上宽泛权限
- capability 文件带 `$schema`(指向 `gen/schemas/desktop-schema.json`)获得补全校验

**CSP 现状与债务(如实记录)**:`tauri.conf.json` 当前 `"csp": null`(开发便利),配置注释已声明「生产建议收紧」。发布流水线(`bun run release` → `release.yml`)已就位,这项债务不再有「还没法发布」作缓冲——**正式发布前必须配置 CSP**(官方基线:`default-src 'self'`,连接放行 `ipc: http://ipc.localhost`);本项目无远程资源加载,收紧成本低。在那之前,不要引入依赖远程脚本 / 远程样式的实现,避免抬高收紧成本。

**IPC 边界校验**:前端传入的数值、枚举在命令层校验(clamp / 类型系统兜底),见[命令与 IPC](./commands-and-ipc.md)。

---

## 配置文件

- `tauri.conf.json` 启用了 json5(`config-json5` feature),**每个配置项写中文注释**说明用途与取值理由(现有文件为范例);`.vscode/settings.json` 已把它关联成 jsonc
- 窗口行为类配置(尺寸、透明、置顶)改动时,检查前端是否有对应的镜像常量或 CSS 假设(宽度 ↔ `WINDOW_WIDTH`,高度上限 ↔ 面板 `max-h`)

---

## 提交前自查

1. `cargo fmt` + `cargo clippy` + `cargo test` 全过
2. 新增命令已注册 `generate_handler!`,前端镜像(`lib/api.ts`)已同步
3. 新增 plugin / 能力的 capability 权限已声明且最小
4. 新增跨端结构体逐字段注释,serde camelCase
5. 业务代码无 `unwrap()`、无 `println!`
