# 状态与并发

> 共享状态只有一个入口(AppState),阻塞操作只有一个去处(spawn_blocking)。

---

## AppState

- 共享状态定义成结构体(`src-tauri/src/state.rs`),setup 里 `app.manage(AppState::new(pool))` 注册一次;命令用 `state: State<'_, AppState>` 注入,服务层经 `app.state::<AppState>()` 或 `app.try_state`(setup 早期可能未注册)获取。**不用全局 static**
- 锁包**字段**不包整个结构体,缩小锁粒度:`paste_target: Mutex<Option<isize>>`、`pending_self_write: Mutex<Option<String>>`,只读的连接池裸放
- 字段不直接暴露,一律经语义化方法访问(`remember_paste_target` / `take_self_write_if_matches`),锁的存在是实现细节
- 锁中毒恢复是既定模式:保存的都是简单值、不存在被破坏的不变量,统一走 `lock_or_recover`(`PoisonError::into_inner`),不 `unwrap` 锁
- **锁不跨 `await` 持有**:现有方法都是同步短临界区,新增字段保持这个形态;需要跨 await 的状态改用 tokio 的异步原语再议

---

## async 与阻塞边界

| 场景 | 做法 | 实例 |
|------|------|------|
| 含 IO / 数据库的命令 | `async fn` 命令 + `.await` | `commands/clipboard.rs` 全部命令 |
| 同步阻塞库(arboard)、Win32 调用、sleep | `tauri::async_runtime::spawn_blocking` | `paste.rs` 的剪贴板写入与焦点还原 |
| setup 里需要 async 初始化 | `tauri::async_runtime::block_on`(仅 setup / RunEvent 等同步上下文) | `lib.rs` 的 `db::init_pool` 与退出时 `pool.close()` |
| 常驻后台异步循环 | `tauri::async_runtime::spawn` | `clipboard_ingest.rs` 的入库循环 |
| 需要 OS 消息循环的常驻监听 | `std::thread::Builder::new().name(...).spawn` | `win_monitor.rs`(线程要命名) |

---

## 采集链路范式(线程 → 通道 → 异步循环)

「平台事件源持续产出、业务侧异步消费」的既定形态(`clipboard_ingest.rs`):

```
平台监听线程(Win32 消息循环) --mpsc::unbounded_channel--> async 入库循环(过滤 → 入库 → 清理 → emit)
```

- 通道两端解耦:平台层只管 `tx.send`,发送失败(接收端关闭)记日志退出线程
- 消费循环里的每次处理错误**记日志继续**,不中断循环(`log::warn!("剪贴板内容入库失败: {err}")`)
- 新增系统事件源(如按键监听、文件监听)沿用同一形态

---

## 回环防护(自写标记)

程序自己写剪贴板会再触发自己的监听,防回环协议由 AppState 承载(`state.rs` + `paste.rs` + `clipboard_ingest.rs` 三处协作):

1. 写入前 `mark_self_write(hash)` 打标
2. 写入失败 `clear_self_write()` 回滚标记,避免误吞下一次真实复制
3. 监听侧 `take_self_write_if_matches(&hash)` 命中即消费标记并跳过入库

任何「自己触发系统事件再被自己监听」的新功能,先套这个标记-消费协议。

---

## 优雅退出

进程退出前在 `RunEvent::Exit` 里显式关闭连接池(等在途写入完成并 checkpoint WAL,`lib.rs`);监听线程与全局快捷键不需要手动清理,随进程回收。新增需要 flush 的资源(未落盘缓存等)时挂到同一个 Exit 分支。
