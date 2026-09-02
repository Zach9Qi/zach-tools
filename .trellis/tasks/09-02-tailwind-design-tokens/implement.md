# 执行计划:Tailwind 设计令牌体系

依赖阅读:`design.md` §2(词表取值)、§3(基础层)、§4(替换映射)。

## Step 1 重写 `src/index.css`

- [ ] `:root` 原始层:按 design §2.1 / §2.2 声明全部原始变量,颜色用 `light-dark()` + `var(--color-zinc-*)`,alpha 用 `--alpha()`
- [ ] `@theme inline` 语义层:`--color-*` 映射、`--radius-*` 派生、`--font-sans/mono` 映射、`--shadow-panel` 映射
- [ ] `@theme`(非 inline):`--text-2xs` 及其行高
- [ ] `@layer base`:design §3 全部规则
- [ ] 每组变量带中文注释:角色 + Radix 档位 + 使用场景(延续现有 index.css 注释风格)
- 验证:`bun run build` 通过;`grep -c -- "--color-zinc-" dist/assets/*.css` ≥ 1(色阶变量未被裁剪;若为 0 走 design §6 回退方案)

## Step 2 组件类名机械替换

- [ ] 以 `grep -rn "surface\|content-\|text-content\|line\|edge\|amber" src --include=*.vue` 为清单,按 design §4 逐处替换
- [ ] `bg-surface-muted` 逐处判定:图标底座 → `bg-muted`;`selected` / `hover:` → `bg-accent`
- [ ] `LauncherPanel.vue` section 移除 `select-none cursor-default antialiased`(已由 base 层承担),并更新该处 HTML 注释中对 `scheme-light-dark` 的说明(仍成立,措辞对齐新 token 名)
- [ ] 不改任何 `<script>`、不改 DOM 结构、不新建组件
- 验证:
  - `grep -rnE "(bg|text|border|ring|fill|placeholder:text)-(surface|content|line|edge|muted-strong)|amber-" src --include=*.vue` 零命中
  - `grep -rnE "text-\[[0-9.]+px\]" src --include=*.vue` 零命中

## Step 3 运行验证

- [ ] `bun run format && bun run lint && bun run test && bun run build` 全过
- [ ] `bun run dev` 浏览器打开:DevTools Rendering 面板分别模拟 `prefers-color-scheme: light / dark`,检查主页(磁贴、键帽、页脚)与剪贴板页(列表、选中态、详情、收藏星)
- [ ] 将 design §5 的换肤样例临时贴进 `index.css` 末尾,确认整体配色 / 圆角变化;再改 `--font-size-base: 87.5%` 确认等比缩放;**验证后删除**
- 回滚点:此处若发现架子缺陷,回到 Step 1 修正,不进入 Step 4

## Step 4 spec 同步

- [ ] 新建 `.trellis/spec/frontend/design-tokens.md`,章节:
  1. 三层架构图与单向数据流(可复用 design §1)
  2. 颜色词表表格:token / 职责 / 何时用 / 不该用于什么(重点:`muted` vs `accent`、`border` vs `input`、`foreground` vs `muted-foreground`)
  3. 圆角 / 字号 / 字体角色约定
  4. 「如何新增一个 token」:先问是否已有档位能覆盖 → 原始层加变量 → 语义层加映射 → 本文登记
  5. 「如何加一套皮肤 / 强制深浅色」:覆盖 `:root` 原始变量;`scheme-light` / `scheme-dark`
  6. 禁止模式:原语色(`zinc-*`/`amber-*`)进组件、`text-[Npx]`、手写 hex 进 `:root`、`dark:` 变体、语义层写具体值、新建 `tailwind.config.js`
- [ ] `component-guidelines.md` 样式章节:改写 token 说明、示例类名(`bg-surface-muted → bg-accent`),链接到 design-tokens.md
- [ ] `index.md`:技术栈表样式行、规范索引加 design-tokens 行、速览的 token 例子改新词表
- [ ] `quality-guidelines.md` 禁止模式表:更新 `dark:` 行的正确做法措辞,新增「原语色 / `text-[Npx]`」行
- [ ] `directory-structure.md`:`index.css` 描述改为「三层 token(原始 / 语义 / base)」
- 验证:`grep -rn "surface\|text-muted\b\|border-line\|ring-edge" .trellis/spec/frontend/` 零命中

## Step 5 收尾

- [ ] 全量 check(`trellis-check`):spec 合规、lint / test / build、`grep` 门禁复核
- [ ] 提交:`refactor(frontend): 按三层 token 方法论重搭 Tailwind 设计令牌体系`(遵循 `.trellis/spec/guides/project-conventions.md`)
- [ ] 归档任务

## 回滚

单 commit;`git revert <sha>` 完整回退,无接口 / 数据影响。
