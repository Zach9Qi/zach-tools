# 质量规范

> 前端改动的验收基线:命令能过、注释到位、错误有出口、禁止模式零出现。

---

## 校验命令

| 时机 | 命令 | 说明 |
|------|------|------|
| 提交前(必须) | `bun run lint` | oxlint,含依赖边界强制(见下);同时覆盖 `scripts/` |
| 提交前(必须) | `bun run typecheck:node` | `tsc --noEmit -p tsconfig.node.json`,Node 侧 TS 的类型门禁:`vite.config.ts` / `vitest.config.ts` / `scripts/**`(`vue-tsc --noEmit` 不跟进 `references`,这些文件不在它的范围内) |
| 提交前(必须) | `bun run test` | Vitest 全量用例 |
| 提交前(必须) | `bun run build` | `vue-tsc --noEmit` 类型门禁 + vite 构建 |
| 提交前(必须) | `bun run format` | Prettier 格式化 `src/ scripts/`(printWidth 100,tailwind 插件自动排类名) |
| CI(自动) | `.github/workflows/ci.yml` | `main` 的 push / PR 依次跑 lint → typecheck:node → test → build,与本地命令一致 |
| 联调 | `bun run tauri dev` | 完整桌面运行时 |
| 纯 UI 预览 | `bun run dev` | 浏览器预览;IPC 层经 `isTauriRuntime()` 全部降级为 no-op,必须保持可打开 |

**Lint(oxlint,配置在 `.oxlintrc.json`)**:检查 TS 与 SFC 的 script 块;`src/tools/**` 的 overrides 用 `no-restricted-imports` 强制「tools 禁止导入 `@/launcher/**`」的依赖方向(gitignore 风格通配,`**` 才覆盖任意深度)。已知缺口(如实记录,知情接受):Vue `<template>` 内部不被 lint(oxlint 上游限制,vue-tsc strict 可兜住模板类型错误,`:key`、`v-html` 等纪律靠评审);类型感知模式(tsgolint)要求 TS 7,项目在 TS ~5.6 暂不启用,故 `no-floating-promises` 缺位,「故意不等待的 Promise 用 `void` 标记」仍靠约定。

**Node 侧 TS(`tsconfig.node.json`)**:`strict` + `types: ["node"]` + `lib: ["ES2022"]`(无 DOM),`scripts/` 下的脚本只用 Node 内置模块、不依赖 Bun 专有 API。它被根 `tsconfig.json` 的 `references` 引用,TS 因此要求它保留 `composite: true` 且**不能在文件里写 `noEmit`**(否则 `vue-tsc --noEmit` 报 TS6306 / TS6310),`noEmit` 只由 `typecheck:node` 命令行传入;`composite` 隐含 `incremental`,`tsBuildInfoFile` 指到 `node_modules/.tmp/` 以免在仓库根落 tsbuildinfo。

**测试(Vitest,配置在根级 `vitest.config.ts`)**:测试文件与被测源码同目录、命名 `xxx.test.ts`、用 `@/` 别名导入;配置独立于 `vite.config.ts`,不牵动 Tauri dev 设置。当前覆盖 `lib/` 纯函数(`tools/clipboard/lib/time.test.ts`、`tools/match.test.ts`),新增纯函数逻辑应同步补用例;纯逻辑尽量下沉为无依赖纯函数(`formatRelativeTime` 带 `now` 参数即为可测设计)。组件测试(@vue/test-utils)尚未引入,是独立决策。

---

## 注释纪律

- 注释一律中文,解释「为什么」与约束,不复述代码;好的参照:`useKeymap.ts` 对 Tab 拦截、输入法组词、Ctrl 组合放行的三段注释
- 导出的接口、类型、函数、常量必须有文档注释;跨端结构体逐字段注释(见[类型安全](./type-safety.md))
- template 里的非显然布局决策用 HTML 注释写在结构旁(`LauncherPanel.vue` 对透明边缘、`max-h` 单一定义处的注释)
- 魔法数字提为带注释的命名常量:`PAGE_SIZE` / `REFRESH_DEBOUNCE_MS`(`useClipboardPage.ts`)、`WINDOW_WIDTH`(`src/lib/window.ts`)、`ROW_CAPACITY`(`useResults.ts`)

---

## 错误处理

- IPC 调用点(composable 的动作函数)用 try/catch 包裹,`console.error` 带中文上下文后**早返回**,不让异常冒泡炸掉 UI;成功路径的后续步骤(本地镜像、反馈点亮)放在 catch-return 之后:

```ts
// src/tools/clipboard/composables/useClipboardPage.ts
/** 粘贴条目;后端负责隐藏窗口、还原焦点并注入 Ctrl+V */
async function paste(item: ClipboardItem) {
  try {
    await pasteClipboardItem(item.id);
  } catch (error) {
    console.error("粘贴失败:", error);
    return;
  }
  mirrorTouch(item);
}
```

- 后端 `AppError` 序列化出来的就是用户可读中文文案,展示时无需再翻译
- 故意不等待的 Promise 用 `void` 显式标记(`void paste(selected.value)`、`void resizeLauncherToContent(...)`),不留裸浮动 Promise
- 环境能力缺失走降级不走报错:非 Tauri 运行时所有 IPC 封装返回 no-op(`lib/runtime.ts` + 各 `lib/api.ts` 的前置判断)

---

## 禁止模式清单

| 禁止 | 正确做法 |
|------|----------|
| 拼接 Tailwind 类名片段(`bg-${color}-500`) | 完整类名间条件切换 |
| 手写 `dark:` 双色维护 | `@theme` 语义 token(`light-dark()`) |
| 组件里直接 `invoke` / `listen` / 字符串命令名 | 经模块 `lib/` 封装函数 |
| `../../` 相对路径导入 | `@/` 别名 |
| 外壳按工具 id 写分支 | 经 `tools/registry.ts` 查表 |
| 创建 `tailwind.config.js` | `src/index.css` 的 `@theme` |
| 引入 lucide 之外的图标库 | `~icons/lucide/*` |
| `any`、滥用 `!` 断言 | 泛型 / 收窄 / 早返回 |
| tools 导入 `@/launcher/*`(反向依赖,oxlint 强制报错) | 外壳能力从共享层(`src/lib/`、`src/composables/`)获取 |
| 单模块私有代码放进根级共享层 | 共享层门槛:≥2 个功能模块真实消费 |

---

## 提交前自查

1. `bun run format && bun run lint && bun run typecheck:node && bun run test && bun run build` 通过
2. 浏览器预览(`bun run dev`)不白屏、无未捕获异常
3. 新增导出都有中文文档注释;跨端类型两侧(`lib/api.ts` ↔ Rust 结构体)已同步
4. 对照上表扫一遍禁止模式
