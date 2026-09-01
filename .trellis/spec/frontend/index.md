# 前端开发规范

> zach-tools 前端(Vue 3 + TypeScript + Tailwind CSS v4)的编码规范。
> 全部规则提炼自真实代码,示例均指向仓库内实际文件,可直接打开对照。

---

## 技术栈

| 维度 | 选型 | 备注 |
|------|------|------|
| 框架 | Vue 3.5+,`<script setup>` 组合式 API | 全仓库无 Options API 代码 |
| 语言 | TypeScript(strict) | 构建即类型门禁:`vue-tsc --noEmit` |
| 样式 | Tailwind CSS v4(CSS-first 配置) | 无 `tailwind.config.js`,主题在 `src/index.css` 的 `@theme` 块 |
| 图标 | unplugin-icons + lucide | `~icons/lucide/xxx` 按需编译成组件 |
| 构建 | Vite 6 + bun | `bun run dev` / `bun run build` |
| 状态 | 组合式函数(模块级 ref) | 未引入 Pinia,见[状态管理](./state-management.md) |
| 桌面运行时 | Tauri 2(@tauri-apps/api) | IPC 封装约定见[目录结构](./directory-structure.md)与[组合式函数规范](./composable-guidelines.md) |

---

## 规范索引

| 文档 | 内容 | 状态 |
|------|------|------|
| [目录结构](./directory-structure.md) | 功能模块划分、工具注册机制、文件归属判断 | 已填写 |
| [组件规范](./component-guidelines.md) | SFC 结构、props/emits 声明、拆分信号、样式写法 | 已填写 |
| [组合式函数规范](./composable-guidelines.md) | useXxx 约定、共享状态、请求编排、事件清理 | 已填写 |
| [状态管理](./state-management.md) | 状态层级、跨组件共享、与后端状态同步 | 已填写 |
| [类型安全](./type-safety.md) | 判别联合、跨端类型镜像、strict 纪律 | 已填写 |
| [质量规范](./quality-guidelines.md) | 校验命令、注释约定、错误处理、禁止模式 | 已填写 |

后端(Rust/Tauri)规范在 [.trellis/spec/tauri/](../tauri/index.md),跨端结构体、命令与事件的两侧约定需要对照阅读。

---

## 一分钟速览(写任何前端代码前默读)

- 代码按**功能模块**组织:启动器外壳在 `src/launcher/`,每个工具在 `src/tools/<id>/`;`src/lib/` 只放真正跨模块的代码
- 组件一律 `<script setup lang="ts">`,props / emits 用类型式声明,禁止 `any`
- 样式只写 Tailwind 工具类,颜色用 `src/index.css` 定义的语义 token(`bg-surface`、`text-muted` 等)
- 与 Tauri 的 `invoke` / `listen` 全部封装在所属模块 `lib/` 里,并用 `isTauriRuntime()` 降级,保证纯浏览器(`bun run dev` 直接打开)也能预览
- 注释用中文、写「为什么」;导出的类型、字段、函数都要有文档注释

---

**文档语言**:本项目规范文档一律使用中文;代码标识符(变量 / 函数 / 类型名)用英文。
