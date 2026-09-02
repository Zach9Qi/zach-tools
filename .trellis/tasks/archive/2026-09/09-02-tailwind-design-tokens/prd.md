# 重构 Tailwind 设计令牌体系

## Goal

按 shadcn/ui + Radix Colors 的三层设计令牌(Design Token)方法论重搭 `src/index.css`,
把"可变性"(配色、字号、圆角)全部收敛到一小撮原始变量,组件只消费语义工具类、不认识任何具体数值。
目标是**搭好架子并写进 spec**,让后续开发按同一套方法论维护;不重写现有组件。

## 背景

对照 EnsoCode(shadcn 生成的 token 体系)分析后,当前 `src/index.css` 的问题:

1. **没有原始层**:`light-dark(#fff, #18181b)` 直接烧在 `@theme` 里,做不了整体换肤、相对派生(改 alpha / 只换强调色)
2. **词表自造且与生态冲突**:`surface / content / line / edge` 是临时命名;`muted` 在本项目是文字色、在 shadcn/Tailwind 生态里是底色
3. **档位失控**:4 档文字、3 档线、5 档圆角散落在 10 个组件里,无角色约定
4. **缺全局入口**:无根字号变量、无字体族 token;`select-none` / `cursor-default` / `antialiased` 等全局策略写在局部组件类名里
5. **有硬编码原语色**:`amber-400` 出现 6 次

## Requirements

### R1 三层 token 架构(`src/index.css`)

- **原始层**:`:root` 上定义原始变量(`--background`、`--foreground`、`--radius`、`--font-size-base` …),颜色值用 `light-dark()` + Tailwind 内置 zinc 色阶变量(`var(--color-zinc-900)`),不再手写 hex
- **语义层**:`@theme inline` 里只做 `--color-background: var(--background)` 这类映射,不出现具体颜色值
- **基础层**:`@layer base` 承载全局行为策略(默认边框色、文本不可选与例外、滚动条、根字号)
- 深浅色切换机制**保留** `light-dark()` + `color-scheme`,不引入 `.dark` 类双份维护
- `color-scheme` 仍挂在面板根(窗口透明边缘不能被画底色),不上 `html`

### R2 token 词表对齐 shadcn 约定

- 颜色角色采用 shadcn 词表:`background / foreground / muted / muted-foreground / accent / accent-foreground / border / input / ring / destructive / destructive-foreground`;可按需扩展(如 `warning`),扩展项须在 spec 中登记
- 文字收敛为两档(Radix 11/12):`foreground`(高对比)、`muted-foreground`(低对比);接受合并带来的轻微色差
- 线收敛为两档:`border`(分隔线、面板外描边)、`input`(键帽、输入框等控件描边)
- 底色分角色:`muted`(图标底座等静态次级底)与 `accent`(hover / 选中态)初始值相同,但**必须**按角色分开使用
- 原语色(`amber-400`、`zinc-*` 等)禁止在组件中直接使用;收藏星这类状态色走语义 token

### R3 圆角 / 字体 / 字号入口

- 圆角单一基准 `--radius`,`sm/md/lg/xl/2xl` 由 `calc` 派生,并约定角色(面板 / 磁贴 / 行 / 图标底座 / 键帽)
- 根字号入口 `--font-size-base`,`html { font-size: var(--font-size-base) }`;默认值保持当前视觉(100%),只是提供入口
- 字体族 token `--font-sans` / `--font-mono`
- 保留 `--text-2xs`(键帽字号);禁止 `text-[Npx]` 任意值,缺档就加 token

### R4 现有组件的机械替换

- 仅做 token 类名替换(`bg-surface → bg-background` 等),不改结构、不抽公共组件、不动 script
- `bg-surface-muted` 按角色分流:图标底座 → `bg-muted`,选中/hover → `bg-accent`

### R5 spec 同步

- 新增 `.trellis/spec/frontend/design-tokens.md`:三层架构图、词表与角色表、圆角/字号角色、如何加 token、如何加一套皮肤、禁止模式
- 更新 `component-guidelines.md` 样式章节、`index.md` 速览与索引、`quality-guidelines.md` 禁止模式表、`directory-structure.md` 对 `index.css` 的描述,示例类名全部改为新词表

## Constraints

- 不新增 npm 依赖(不引入 cva / clsx / tailwind-merge)
- 不创建 `tailwind.config.js`
- 不引入 JS 侧主题切换逻辑(换肤入口留在 CSS 变量上,JS 接入是后续独立任务)
- 面板视觉允许轻微色差,但布局尺寸不变(默认根字号不变)

## Acceptance Criteria

- [ ] `src/index.css` 具备 `:root` 原始层、`@theme inline` 语义层、`@layer base` 三段,语义层不出现任何 hex / oklch / `light-dark()` 具体值
- [ ] 组件源码中 `grep -E "(surface|content|line|edge|amber)"` 零命中(排除注释中的历史说明)
- [ ] 组件源码中 `text-\[[0-9.]+px\]` 零命中
- [ ] 全部颜色原始变量以 `light-dark()` + `var(--color-zinc-*)`/`var(--color-white)` 定义(允许 alpha 用 `--alpha()` 或 `color-mix()`)
- [ ] 只在 `:root` 覆盖 `--background` 等原始变量即可整体换肤(在 design.md 中给出一段可粘贴验证的覆盖样例,实施时用浏览器 DevTools 验证)
- [ ] 修改 `--font-size-base` 后面板全部尺寸等比缩放(DevTools 验证)
- [ ] `bun run format && bun run lint && bun run test && bun run build` 全过
- [ ] `bun run dev` 浏览器预览下浅色 / 深色(切系统主题或 DevTools 模拟 `prefers-color-scheme`)面板均正常
- [ ] `.trellis/spec/frontend/design-tokens.md` 存在,且 `index.md` 索引到它;其余 spec 中旧词表零残留
