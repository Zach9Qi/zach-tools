# 命令与 IPC

> 前后端通信只有两条路:前端主动调后端走 command,后端通知前端走 event,方向不混用。

---

## 命令定义

- `#[tauri::command]` + snake_case 命名;命令函数保持薄:取参校验 → 调服务 → 返回(`src-tauri/src/commands/clipboard.rs` 全文是基准样板)
- 可失败命令一律返回 `Result<T, AppError>`;无失败路径的才返回裸值(`hide_launcher`)
- 全部命令在 `lib.rs` 的 `tauri::generate_handler![]` 注册,漏注册前端调用会直接报错
- 需要窗口 / 事件能力的命令收 `app: AppHandle<R>` 且函数带 `R: Runtime` 泛型;只碰状态的命令收 `state: State<'_, AppState>` 即可
- 含 IO 的命令声明为 `async fn`,不阻塞主线程

**前端是不可信输入源**(Tauri 官方安全模型):数值范围、枚举合法性等校验放在命令层边界,不指望前端自觉。实例:

```rust
// src-tauri/src/commands/clipboard.rs — list_clipboard_items 结尾
) -> Result<Vec<ClipboardItem>, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let filter = ListFilter {
        keyword: query.as_deref(),
        kind,
        favorite_only: favorite_only.unwrap_or(false),
    };
    Ok(store::list(state.db(), filter, limit, cursor).await?)
}
```

- 多命令共用的取数逻辑抽成模块内私有辅助函数(`get_content`:双层 Option 区分「不存在」与「类型不支持」,分别映射成 `ItemNotFound` / `UnsupportedKind`;返回的 `ClipContent` 交给 `paste::copy_to_clipboard` 按文本 / 图片分派,命令层不认识内容类型的差异)

---

## 事件(后端 → 前端)

- 事件名 kebab-case,定义为模块顶部常量,不写裸字符串:`EVENT_OPEN = "launcher-open"`(`launcher_window.rs`)、`EVENT_NEW_ITEM = "clipboard-new-item"`(`clipboard_ingest.rs`)
- 用 `app.emit(EVENT, payload)` 广播;前端在对应的 `lib/` 封装里有同名监听函数(窗口事件 → `src/lib/window.ts`,剪贴板 → `tools/clipboard/lib/api.ts`),新增事件两侧同时加
- 事件语义要考虑「前端可能错过事件」:剪贴板重复复制时后端以同一 id 重发条目,前端按 id 去重;窗口唤起时前端做首条比对补偿(见 frontend spec 状态管理篇)

---

## 跨端结构体

- 统一 `#[derive(Serialize)]`(需要反序列化再加 `Deserialize`)+ `#[serde(rename_all = "camelCase")]`;字符串枚举加 `#[serde(rename_all = "lowercase")]`(`ClipboardKind`)
- **每个字段写中文文档注释**,类型专属字段标注适用范围(`/// [image] 原图落盘路径`),前端 `lib/api.ts` 的镜像接口逐字段对应(参照 `ClipboardItem` ↔ `api.ts` 的 `ClipboardItem`)
- 改动跨端结构体的 checklist:后端结构体 → 前端镜像接口 → 两侧注释 → 序列化值(枚举)三处同步

---

## IPC 载荷纪律

大内容不过 IPC,这是既定架构决策(`clipboard_store.rs` 模块注释):

- 列表与事件载荷统一走「预览投影」:文本截断到 `PREVIEW_MAX_CHARS`(5000 字符)并附原文总长,原文不出库
- 原文按 id 用单独命令现取(粘贴 / 复制时后端内部取用,不回传前端)
- 大二进制(图片)落盘存路径,库和 IPC 只走路径字符串;前端经 `toAssetUrl`(`convertFileSrc` 封装,`lib/api.ts`)用 asset 协议按需加载,**不走 base64 过 IPC**。可读范围由 `tauri.conf.json5` 的 `assetProtocol.scope`(仅 `$APPLOCALDATA/clipboard-images/**`)管控

新增「列表 + 详情」类功能时沿用:列表载荷带预览与元数据,重内容按 id 二次获取或后端内部消化。

---

## 新增一个命令的完整路径

1. 服务层实现业务函数(`services/`,能纯则纯)
2. `commands/<domain>.rs` 加薄命令:文档注释(语义、参数含义)+ 参数校验 + 调服务
3. `lib.rs` 的 `generate_handler![]` 注册
4. 前端所属模块 `lib/api.ts` 加封装函数(带 `isTauriRuntime()` 降级)与类型镜像
5. 需要新权限(plugin / core 能力)时在 `capabilities/default.json` 声明最小权限
