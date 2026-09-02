# 技术设计:Tailwind 设计令牌体系

## 1. 架构:三层 + 一条单向数据流

```
:root                      原始层(Primitive/Alias 合并)——唯一允许出现具体值的地方
  --background: light-dark(var(--color-white), var(--color-zinc-900));
  --radius: 0.5rem;  --font-size-base: 100%;
        │  var()
@theme inline              语义层——只做映射与派生,零具体颜色值
  --color-background: var(--background);
  --radius-md: calc(var(--radius) - 2px);
        │  工具类
组件                        bg-background / rounded-md / text-sm——不认识任何数值
        ↑
@layer base                基础层——全局行为策略(根字号、默认边框色、可选性、滚动条)
```

原则:
- **可变性只从 `:root` 进入**:换肤 = 覆盖 `:root` 一批原始变量;调字号 = 改 `--font-size-base`
- **派生不复制**:圆角档位由 `--radius` 算出;`muted` 与 `accent` 初值相同但各自独立声明(角色不同,允许日后分叉)
- **`@theme inline`** 而非 `@theme`:让工具类直接输出 `var(--background)`,这样在任意子树(而非仅 `:root`)覆盖原始变量都能生效,是换肤能力的前提
- **深浅色仍靠 `light-dark()` + `color-scheme`**:`light-dark()` 在消费元素处求值,面板根上的 `scheme-light-dark` 决定走哪支;强制浅/深只需把该类换成 `scheme-light` / `scheme-dark`,无需 `.dark` 双份维护

## 2. 词表与取值(zinc 基准)

### 2.1 颜色(shadcn 词表,Radix 档位职责)

| 语义 token | Radix 档位/职责 | 浅色 | 深色 | 替换自 |
|---|---|---|---|---|
| `background` | 1-2 面板底 | `white` | `zinc-900` | `surface` |
| `foreground` | 12 高对比文字 | `zinc-800` | `zinc-100` | `content`、`content-secondary` |
| `muted` | 3 静态次级底(图标底座、键帽底) | `zinc-100` | `zinc-800` | `surface-muted`(底座用法) |
| `muted-foreground` | 11 低对比文字(提示、时间、占位、键帽字) | `zinc-500` | `zinc-400` | `muted`、`muted-strong` |
| `accent` | 4-5 交互态底(hover / 选中) | `zinc-100` | `zinc-800` | `surface-muted`(选中用法) |
| `accent-foreground` | 交互态上的文字 | `zinc-800` | `zinc-100` | 新增 |
| `border` | 6 分隔线、面板外描边 | `zinc-200 / 70%` | `zinc-800` | `line`、`edge` |
| `input` | 7 控件描边(键帽、输入框) | `zinc-300 / 80%` | `zinc-700` | `line-strong` |
| `ring` | 8 焦点环 | `zinc-400` | `zinc-500` | 新增 |
| `destructive` | 危险操作 | `red-600` | `red-400` | 新增 |
| `destructive-foreground` | | `white` | `white` | 新增 |
| `warning` | 高亮/收藏/告警 | `amber-500` | `amber-400` | 硬编码 `amber-400` |
| `warning-foreground` | | `zinc-900` | `zinc-900` | 新增 |

取值全部引用 Tailwind 内置 oklch 色阶(`var(--color-zinc-900)`),alpha 用 `--alpha(var(--color-zinc-200) / 70%)`(Tailwind 4.1+ 编译期函数)。不手写 hex。

`content-secondary → foreground` 与 `muted-strong → muted-foreground` 是有意收敛(用户已接受色差)。

### 2.2 非颜色 token

| token | 原始变量 | 值 | 说明 |
|---|---|---|---|
| `--radius-sm/md/lg/xl/2xl` | `--radius: 0.5rem` | `-4px / -2px / 0 / +4px / +8px` | 与当前 4/6/8/12/16px 完全一致,组件圆角类名不变 |
| `--font-sans` | `--font-family-sans` | `system-ui, -apple-system, "Segoe UI", sans-serif` | Preflight 自动作用于 `html` |
| `--font-mono` | `--font-family-mono` | `ui-monospace, "Cascadia Mono", Consolas, monospace` | 剪贴板文本预览后续可用 |
| `--text-2xs` | (直接在 `@theme`) | `0.625rem / lh 1` | 保留;字号属刻度而非皮肤,允许直接写值 |
| `--shadow-panel` | `--panel-shadow` | `0 12px 32px -8px rgb(0 0 0 / 0.35)` | 保留 |
| `html { font-size }` | `--font-size-base: 100%` | | 根字号入口;默认不改变现有视觉 |

圆角角色约定(写入 spec):面板 `2xl`、磁贴 `xl`、行/卡片/图标底座(大) `lg`、图标底座(小) `md`、键帽 `sm`。

## 3. 基础层(`@layer base`)

从组件类名上收回的全局策略:

```css
html { font-size: var(--font-size-base); }
body { cursor: default; -webkit-font-smoothing: antialiased; }
* { border-color: var(--color-border); }           /* 写 border 不必再带颜色 */
* { user-select: none; }                            /* 启动器为非文档型 UI */
input, textarea, [contenteditable="true"], .select-text, .select-text * { user-select: text; }
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-thumb { background: --alpha(var(--muted-foreground) / 30%); border-radius: 9999px; }
::-webkit-scrollbar-track { background: transparent; }
```

注意:`body` **不**设 `background` / `color`——窗口透明边缘不能有底色,且 `light-dark()` 在无 `color-scheme` 的 `body` 上只会走浅色支。面板根继续显式 `bg-background text-foreground`。

`LauncherPanel.vue` 的 section 上 `select-none cursor-default antialiased` 因此变为冗余,一并移除(属机械清理,不改结构)。

## 4. 组件替换映射(纯类名替换)

| 旧 | 新 | 备注 |
|---|---|---|
| `bg-surface` | `bg-background` | |
| `bg-surface-muted`(图标底座 `<span class="flex size-* ...">`) | `bg-muted` | ToolTile、ClipboardListItem、空态图标 |
| `bg-surface-muted`(`selected ? ... : ''`、`hover:`) | `bg-accent` | 选中/悬停 |
| `text-content`、`text-content-secondary` | `text-foreground` | |
| `text-muted`、`text-muted-strong` | `text-muted-foreground` | |
| `border-line`、`bg-line`(分隔线) | `border-border`、`bg-border` | |
| `border-line-strong` | `border-input` | KeyboardKey |
| `ring-edge` | `ring-border` | LauncherPanel |
| `fill-amber-400 text-amber-400` | `fill-warning text-warning` | 收藏星 |
| `placeholder:text-muted` | `placeholder:text-muted-foreground` | SearchInput |

实施时以 `grep -rn "surface\|content\|line\|edge\|amber" src --include=*.vue` 为清单,逐处判断底座 / 交互态归属。

## 5. 换肤与字号:验证样例

粘贴到 DevTools 的 `<style>` 或临时加在 `index.css` 末尾即可验证架子生效(验证后删除):

```css
/* 换肤:只碰原始层 */
:root {
  --background: light-dark(var(--color-stone-50), var(--color-stone-900));
  --foreground: light-dark(var(--color-stone-800), var(--color-stone-100));
  --muted: light-dark(var(--color-stone-200), var(--color-stone-800));
  --accent: light-dark(var(--color-amber-100), var(--color-amber-950));
  --border: light-dark(var(--color-stone-300), var(--color-stone-700));
  --radius: 0.25rem;
}
/* 字号:全面板等比缩放 */
:root { --font-size-base: 87.5%; }
```

预期:面板配色、圆角整体变化,所有尺寸(图标、行高、面板上限 `max-h-128`)随根字号缩放,组件零改动。

## 6. 取舍与风险

| 取舍 | 决策 | 原因 |
|---|---|---|
| `.dark` 类 vs `light-dark()` | `light-dark()` | 零 JS、零双份;Tailwind v4 官方亦推荐。代价:强制模式要改面板根的 `scheme-*` 类,而非 `html` |
| 原始层放 `:root` 还是面板根 | `:root` | 与 shadcn 一致,换肤覆盖点唯一;`light-dark()` 在消费处求值,所以变量声明位置不影响深浅色判断 |
| 是否引入 JS 主题切换 | 不 | 本任务只搭 CSS 架子;JS 写 `style.setProperty` 是后续任务,接口就是 `:root` 变量名 |
| 引用 `var(--color-zinc-*)` 是否会被 Tailwind 裁剪 | 预期不会(Tailwind 追踪 CSS 内 `var()` 引用) | **风险点**:实施时 `grep -- "--color-zinc-900" dist/assets/*.css` 验证;若被裁剪,回退为在 `@theme static` 中声明所需色阶 |
| 语义层禁止具体值,`--text-2xs` 例外 | 允许 | 字号刻度不属于"皮肤",无换肤意义 |
| `content-secondary` 归 `foreground` 还是 `muted-foreground` | `foreground` | 磁贴标题、空态标题是主要内容,对应 Radix 12;shadcn Empty 组件同样标题用 `foreground` |

## 7. 兼容与回滚

- 视觉:尺寸、布局零变化;颜色有轻微色差(用户已接受)
- 回滚:单 commit,`git revert` 即可;无数据、无接口变更
