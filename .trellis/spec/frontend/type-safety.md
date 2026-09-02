# 类型安全

> tsconfig 已开 strict。类型的职责不止「不报错」,还要把**非法状态挡在编译期**。

---

## 基线纪律

- 禁止 `any`(全仓库现无一处);未知形状用泛型或 `unknown` + 收窄
- 不滥用非空断言 `!`:优先可选链、`??`、早返回、`find` 后判空;查不到属于编程错误时抛带上下文的 Error(`registry.ts` 的 `moduleOf`)
- 类型导入用 `import type` 或行内 `type` 修饰(`import { listen, type UnlistenFn } from "@tauri-apps/api/event"`)
- 公开接口 / 类型的每个字段写中文文档注释,说明含义、单位、边界(全部现有接口如此)

---

## 用判别联合让非法状态不可表示

工具注册单元是本项目的基准范式(`src/tools/types.ts`):

- `ToolModule = ViewToolModule | LaunchToolModule`,以 `item.action` 为判别字段
- 每个成员用 `never` 封死不该有的字段——view 型 `run?: never`,launch 型 `page?: never`,「view 型才有页面、launch 型才有动作」由编译器保证,误配在编译期报错
- 判别字段嵌套在 `item.action` 上,TS 不会自动收窄,统一走谓词函数:

```ts
// src/tools/types.ts
/** ToolModule 联合的收窄谓词(嵌套判别字段 TS 不会自动收窄,统一走这里) */
export function isViewModule(module: ToolModule): module is ViewToolModule {
  return module.item.action === "view";
}
```

新增「多形态注册单元 / 配置项」时套用这套写法,不要用可选字段 + 运行时 if 兜底。

---

## 跨端(IPC)类型镜像

与 Rust 后端传输的类型集中在模块 `lib/api.ts`,逐字段镜像后端结构体(对照 `src-tauri/src/services/clipboard_store.rs`):

- 字段名 camelCase,与后端 `#[serde(rename_all = "camelCase")]` 一致;字符串枚举与后端 `rename_all = "lowercase"` 序列化值一致(`ClipboardKind = "text" | "image" | "files"`)
- 后端 `Option<T>` → 前端 `T | null`(不是 `?:`),明确「后端会给 null」与「字段可缺省」的区别;命令入参的可缺省字段才用 `?:`(`ListClipboardParams`)
- 每个字段的文档注释与后端保持同步,类型专属字段标注适用 kind(`/** [image] 原图落盘路径 */`)
- 事件载荷用 `listen<ClipboardItem>(...)` 显式标注(`api.ts` 的 `onClipboardNewItem`)

**改动后端跨端结构体时,必须同步修改对应 `lib/api.ts` 的镜像类型与注释**,两侧规范互为对照(后端侧见 `.trellis/spec/tauri/commands-and-ipc.md`)。

---

## 其他既定模式

- 字符串 → 组件映射用 `Record<string, Component>` + 带兜底的查询函数,不在数据结构里存组件类型(`src/tools/icons.ts`)
- 联合字面量做受限枚举:`type SectionKey = "recent" | "pinned" | "matches" | "named"`(`useResults.ts`)、`ToolActionKind = "view" | "launch"`
- 函数参数用最窄类型:方向增量写 `delta: 1 | -1`(`useRowNavigation.ts`),不写 `number`
- 泛型 composable 的类型参数只约束必要形状(`useRowNavigation<T>` 对 T 零约束,行切分方式由调用方决定)

---

## 校验

类型检查就是构建门禁的一部分,提交前必须通过:

```bash
bun run build   # vue-tsc -b && vite build
```
