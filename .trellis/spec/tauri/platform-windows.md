# Windows 平台层

> 所有直接碰系统 API 的代码收在 `platform/`,业务层只见 facade 导出的普通函数,看不见 Win32。

---

## facade 结构

`src-tauri/src/platform.rs` 是唯一入口:

- 平台无关的数据类型定义在 facade(`ClipboardCapture`)
- `#[cfg(windows)]` 声明真实实现模块并 `pub use` 逐个导出函数;`#[cfg(not(windows))]` 导出 `stub.rs` 的同签名空实现,**保证任何平台都能编译**
- 业务层仅 Windows 才有意义的钩子（如 `install_platform_hooks`）：函数和调用点都加 `#[cfg(windows)]`，非 Windows 不提供空实现。不要写“同签名但函数体整段 `#[cfg(windows)]`”，那会在 Ubuntu CI 报 `unused_variables`；若确实需要跨平台同签名，走 facade/stub 模式，stub 参数用 `_param` 命名
- stub 的空实现不是静默:有副作用缺失的记 `log::warn!`(`spawn_monitor`),纯查询返回中性值(`foreground_window` → `None`,`focus_window` → `false`)
- 新增平台能力三步:facade 定义签名 → `win_xxx.rs` 实现 → `stub.rs` 补同签名空实现

跨层传递窗口句柄用 `isize`(`HWND.0 as isize`),不让 Win32 类型泄出平台层(`state.rs` 的 `paste_target` 存 isize,`win_input.rs` 负责来回转换)。

---

## Win32 惯例(现有实现提炼)

- **常驻监听线程**:`std::thread::Builder::new().name("clipboard-monitor")` 命名线程;线程函数只包一层错误日志,消息循环体单独成函数(`win_monitor.rs` 的 `spawn_monitor` / `run_message_loop` 分工)
- **message-only 窗口**:只收消息不可见的监听场景用 `HWND_MESSAGE` 父窗口注册窗口类 + `WM_XXX` 消息循环
- **事件去抖**:系统可能对一次动作发多条消息,用系统提供的序列号判重(`GetClipboardSequenceNumber`)
- **资源占用重试**:剪贴板等共享资源被占用时小退避线性重试(`READ_RETRIES = 3`,`READ_RETRY_DELAY * attempt`),重试耗尽返回 None 记 debug 日志,不报错
- **按键注入**:`SendInput` 一次提交完整按下/抬起序列;返回值与预期条数比对,不足记 warn(`win_input.rs` 的 `send_ctrl_v`)
- 失败路径一律**日志 + 降级**,平台层不 panic:焦点还原失败照样注入粘贴(`paste.rs` 对 `focus_window` 返回 false 的处理)

---

## unsafe 纪律

- `unsafe` 块尽量小,只包住 FFI 调用本身;整个函数体确实全是 FFI 时才包函数体(`run_message_loop`)
- Win32 错误码转换用 `windows::core::Error::from_thread()`(`RegisterClassW` 返回 0 的分支)
- 回调(`wnd_proc`)保持最小实现,默认转发 `DefWindowProcW`
- 宽字符串字面量用 `w!` 宏,不手动编码

---

## 依赖声明

windows crate 按 feature 精确引入,只开用到的模块(`Cargo.toml` 的 `[target."cfg(windows)".dependencies]`);新用一个 Win32 API 先查它所属 feature,不整包引入。
