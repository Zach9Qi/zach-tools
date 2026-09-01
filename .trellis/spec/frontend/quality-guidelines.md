# 质量规范

> 前端改动的验收基线:命令能过、注释到位、错误有出口、禁止模式零出现。

---

## 校验命令

| 时机 | 命令 | 说明 |
|------|------|------|
| 提交前(必须) | `bun run build` | `vue-tsc --noEmit` 类型门禁 + vite 构建 |
| 提交前(必须) | `bun run format` | Prettier 格式化 `src/`(printWidth 100,tailwind 插件自动排类名) |
| 联调 | `bun run tauri dev` | 完整桌面运行时 |
| 纯 UI 预览 | `bun run dev` | 浏览器预览;IPC 层经 `isTauriRuntime()` 全部降级为 no-op,必须保持可打开 |

**测试现状(如实记录)**:前端尚未配置测试框架(无 vitest / jest)。当前的补偿手段是:纯逻辑尽量下沉到 `lib/` 的无依赖纯函数(`tools/clipboard/lib/time.ts` 的 `formatRelativeTime` 带 `now` 参数即为可测设计),复杂状态机集中在 composable 且注释完备。引入测试框架是独立决策,不要在业务任务里顺手加。

---

## 注释纪律

- 注释一律中文,解释「为什么」与约束,不复述代码;好的参照:`useKeymap.ts` 对 Tab 拦截、输入法组词、Ctrl 组合放行的三段注释
- 导出的接口、类型、函数、常量必须有文档注释;跨端结构体逐字段注释(见[类型安全](./type-safety.md))
- template 里的非显然布局决策用 HTML 注释写在结构旁(`LauncherPanel.vue` 对透明边缘、`max-h` 单一定义处的注释)
- 魔法数字提为带注释的命名常量:`PAGE_SIZE` / `REFRESH_DEBOUNCE_MS`(`useClipboardPage.ts`)、`WINDOW_WIDTH`(`launcher/lib/window.ts`)、`ROW_CAPACITY`(`useResults.ts`)

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
| 新建根级 `src/components/`、`src/composables/` 堆放处 | 归属到功能模块内 |

---

## 提交前自查

1. `bun run format && bun run build` 通过
2. 浏览器预览(`bun run dev`)不白屏、无未捕获异常
3. 新增导出都有中文文档注释;跨端类型两侧(`lib/api.ts` ↔ Rust 结构体)已同步
4. 对照上表扫一遍禁止模式
