# Rust / Tauri 后端开发规范

> zach-tools 后端(`src-tauri/`,Rust + Tauri 2)的编码规范。
> 全部规则提炼自真实代码,示例均指向仓库内实际文件。

---

## 技术栈

| 维度 | 选型 | 备注 |
|------|------|------|
| 框架 | Tauri 2(`config-json5` + `tray-icon` feature) | 桌面常驻托盘应用,窗口默认隐藏 |
| 数据库 | sqlx 0.9 + SQLite(WAL) | 内嵌 migration,连接池经 AppState 共享 |
| 错误 | thiserror | 统一 `AppError`,序列化为中文文案供前端直接展示 |
| 日志 | log + tauri-plugin-log | debug 构建 Debug 级,发布 Info 级 |
| 剪贴板 | arboard | 读写均在 `spawn_blocking` 中执行 |
| 平台 API | windows crate(Win32) | 隔离在 `platform/`,其余平台走 stub |
| 内容去重 | blake3 | 文本 hash 作全局去重键与自写标记 |

## 分层(调用方向自上而下)

```
commands/   命令层:IPC 入口,薄——取参校验 → 调服务 → 返回
services/   服务层:业务编排(可依赖 tauri 类型)与纯存储(不依赖 tauri,可单测)
platform/   平台层:Win32 等系统 API,cfg 隔离 + stub 兜底
db / state / error   基础设施:连接池初始化、共享状态、统一错误
```

---

## 规范索引

| 文档 | 内容 | 状态 |
|------|------|------|
| [模块组织](./module-organization.md) | 入口职责、目录模块写法、分层边界 | 已填写 |
| [命令与 IPC](./commands-and-ipc.md) | 命令定义、事件方向、跨端结构体、载荷纪律 | 已填写 |
| [状态与并发](./state-and-concurrency.md) | AppState、锁粒度、async 与 spawn_blocking、采集链路 | 已填写 |
| [数据库](./database.md) | sqlx 惯例、keyset 分页、migration、存储层测试 | 已填写 |
| [Windows 平台层](./platform-windows.md) | cfg 隔离、消息循环、unsafe 纪律 | 已填写 |
| [质量与安全](./quality-and-security.md) | fmt/clippy/test 门禁、unwrap 政策、capability 与 CSP | 已填写 |

前端规范在 [.trellis/spec/frontend/](../frontend/index.md);跨端结构体、命令与事件在两侧的约定互为镜像,改一侧必须对照另一侧。

---

## 一分钟速览(写任何后端代码前默读)

- 命令保持薄,业务进 services;能不依赖 tauri 类型的逻辑(如存储层)就不依赖,换取可单测性
- 可失败命令一律 `Result<T, AppError>`;错误文案面向用户、写中文
- 前端调后端用 command,后端通知前端用 `app.emit` + kebab-case 事件名常量,不混用
- 跨端结构体 `#[serde(rename_all = "camelCase")]` + 逐字段中文文档注释,前端 `lib/api.ts` 同步镜像
- 业务代码禁止 `unwrap()`;明确可忽略的 `Result` 用 `let _ =` 显式丢弃
- 新增 plugin / 系统能力时在 `capabilities/` 同步声明最小权限

提交前:`cargo fmt` + `cargo clippy` 无警告,`cargo test` 通过(均在 `src-tauri/` 下执行)。

---

**文档语言**:本项目规范文档一律使用中文;代码标识符用英文。
