# 设计令牌

> 颜色、圆角、字号的可变性只从 `:root` 进入;组件只消费语义工具类,不认识任何具体数值。
> 实现落在 `src/index.css`。换肤 / 调字号只覆盖原始变量,不改组件。

---

## 1. 三层架构与单向数据流

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
- **`@theme inline`** 而非 `@theme`:工具类直接输出 `var(--background)`,任意子树覆盖原始变量都能生效
- **深浅色靠 `light-dark()` + `color-scheme`**:`light-dark()` 在消费元素处求值,面板根的 `scheme-light-dark` 决定走哪支;强制浅/深把该类换成 `scheme-light` / `scheme-dark`,不维护 `.dark` 双份
- **`color-scheme` 只挂面板根**,不上 `html` / `body`(窗口透明边缘不能被画底色;`body` 无 `color-scheme` 时 `light-dark()` 只会走浅色支)
- 字号刻度(`--text-2xs`)不属于皮肤,写在非 inline 的 `@theme` 里,允许直接写值

---

## 2. 颜色词表

**颜色怎么写**(`:root` 里的底 / 字 / 线 / 阴影色 / 滚动条色都适用):

1. **色相**只用 Tailwind 内置变量:`var(--color-zinc-900)` / `var(--color-white)` / `var(--color-black)` / `var(--color-amber-400)` …
2. **透明**用 `--alpha(var(--color-zinc-200) / 70%)`
3. **深浅不同**用 `light-dark(浅色支, 深色支)`
4. **深浅相同**直接写一支,如 `var(--color-white)`,不包空的 `light-dark()`

阴影的偏移和模糊可以是 `px`,其中的颜色仍走上面四步。字面颜色(`#fff` / `oklch(...)` / `rgb(...)` / `hsl(...)`)一律不写。

| token | 职责(Radix 档位) | 何时用 | 不该用于什么 |
|------|------------------|--------|--------------|
| `background` | 1-2 面板底 | 面板根 `bg-background` | 图标底座、选中态 |
| `foreground` | 12 高对比文字 | 主文案、磁贴标题、空态标题、搜索输入 | 提示 / 时间 / 占位 / 键帽字 |
| `muted` | 3 静态次级底 | 图标底座、键帽底、工具徽章等**非交互**次级底 `bg-muted` | hover / 选中(那是 `accent`) |
| `muted-foreground` | 11 低对比文字 | 提示、时间、占位符、键帽字、分区标题 | 主文案;也不要把底色 token 当成字色 |
| `accent` | 4-5 交互态底 | `selected` / `hover:bg-accent` | 静态底座 |
| `accent-foreground` | 交互态上的文字 | 需要与 `accent` 成对出现的字色 | 静态主文案(用 `foreground`) |
| `border` | 6 分隔线、面板外描边 | `border-border`、`bg-border`(竖分割)、`ring-border` | 键帽 / 输入框描边(那是 `input`) |
| `input` | 7 控件描边 | 键帽、输入框 `border-input` | 分区分隔线、面板外环 |
| `ring` | 8 焦点环 | 焦点可见性 `ring-ring` | 面板常驻外环(用 `border`) |
| `destructive` / `destructive-foreground` | 危险操作 | 删除等破坏性动作 | 普通强调 |
| `warning` / `warning-foreground` | 高亮 / 收藏 / 告警(扩展项) | 收藏星 `fill-warning text-warning` | 原语色 `amber-*` |

**`muted` vs `accent`**:初值相同,但必须按角色分开写。底座 / 键帽 → `bg-muted`;选中 / hover → `bg-accent`。日后换肤只改其一即可分叉。

**成对规则**:只约束 `accent` / `destructive` / `warning`——这些 `-foreground` 是「坐在该底色上的字」。同一元素写了 `bg-accent` 就必须配 `text-accent-foreground`,不要用 `text-foreground` 顶替(初值相同也会让 token 变成死的,换肤才露馅)。`muted` / `background` **不受此约束**:`muted-foreground` 是低对比文字,不是「muted 底上的字」。徽章这类次级底上的主文案用 `bg-muted text-foreground`(`ToolSearchBar.vue`);键帽字要淡才用 `text-muted-foreground`。子元素坐在自己的底上(如图标底座)继续按自身角色选字色,不跟外层 accent 绑。

**`border` vs `input`**:分隔与外轮廓走 `border`;可交互控件描边走 `input`。

**`foreground` vs `muted-foreground`**:只保留 Radix 12 / 11 两档。标题、正文用前者;提示、元信息、占位用后者。

---

## 3. 圆角 / 字号 / 字体角色

**圆角**:单一基准 `--radius`(默认 `0.5rem`),档位由 `calc` 派生,组件类名不变。偏移是 `px`(保 2px 台阶),不随根字号等比;只有基准本身是 rem。

| 类名 | 派生 | 像素(默认基准) | 角色 |
|------|------|----------------|------|
| `rounded-sm` | `--radius - 4px` | 4px | 键帽 |
| `rounded-md` | `--radius - 2px` | 6px | 图标底座(小)、详情操作按钮 |
| `rounded-lg` | `--radius` | 8px | 行 / 卡片 / 图标底座(大) / tab |
| `rounded-xl` | `--radius + 4px` | 12px | 磁贴 |
| `rounded-2xl` | `--radius + 8px` | 16px | 面板、空态图标底座 |

**字号**:

- 根入口 `--font-size-base`(默认 `100%`),`html { font-size: var(--font-size-base) }`。改此值后**间距 / 字号 / rem 尺寸**(如 `max-h-128`、图标 `size-*`)等比缩放;圆角档位偏移、`shadow-panel`、滚动条宽度是像素级微调,不跟着缩
- 键帽用 `text-2xs`(`0.625rem` / 行高 1);禁止 `text-[Npx]`,缺档就在 `@theme` 加 token
- 其余用 Tailwind 默认刻度(`text-xs` / `text-sm` / `text-lg` …)

**字体**:`--font-sans`(系统 UI 栈,Preflight 作用于 `html`)、`--font-mono`(等宽,剪贴板文本预览等后续可用)。原始值在 `:root` 的 `--font-family-sans` / `--font-family-mono`。

**投影**:`shadow-panel` 映射 `--panel-shadow`。偏移 / 模糊用 `px`(延伸控制在窗口 20px 透明边内);颜色写成 `light-dark(--alpha(var(--color-black) / 20%), --alpha(var(--color-black) / 50%))`,浅淡深重,和别的 token 同一套路。

---

## 4. 如何新增一个 token

1. **先问是否已有档位能覆盖**。文字只有 `foreground` / `muted-foreground`;底只有 `background` / `muted` / `accent`;线只有 `border` / `input`。角色对得上就复用,不要为「稍微深一点」再开一档
2. **原始层加变量**:按 §2「颜色怎么写」四步声明。示例:`--foo: light-dark(var(--color-zinc-100), var(--color-zinc-800))`;要透明就包 `--alpha(...)`
3. **语义层加映射**:`@theme inline` 里 `--color-foo: var(--foo);`(或 `--radius-*` / `--font-*` / `--shadow-*` 的对应命名空间)
4. **本文登记**:补进上面的词表或角色表,写清职责、何时用、不该用于什么。扩展色(如 `warning`)必须在本文件出现,不能只活在 CSS 里

字号刻度例外:直接写在非 inline 的 `@theme`(如 `--text-2xs`),不必走 `:root`。

---

## 5. 如何加一套皮肤 / 强制深浅色

**换肤**:只覆盖 `:root` 原始变量,不要改 `@theme inline`、不要改组件。

```css
:root {
  --background: light-dark(var(--color-stone-50), var(--color-stone-900));
  --foreground: light-dark(var(--color-stone-800), var(--color-stone-100));
  --muted: light-dark(var(--color-stone-200), var(--color-stone-800));
  --accent: light-dark(var(--color-amber-100), var(--color-amber-950));
  --border: light-dark(var(--color-stone-300), var(--color-stone-700));
  --radius: 0.25rem;
}
```

**调字号**:`--font-size-base: 87.5%;`。间距 / 字号 / rem 尺寸等比缩放;圆角 `± Npx` 与阴影保持像素级微调(阴影绑着窗口 20px 透明边,不宜跟字号跑)。

**强制浅 / 深**:改面板根的 `scheme-light-dark` 为 `scheme-light` 或 `scheme-dark`。不要引入 `.dark` 类,不要写 `dark:` 变体。JS 主题切换(后续任务)的接口就是这些 `:root` 变量名,用 `style.setProperty` 即可。

面板根继续显式 `bg-background text-foreground scheme-light-dark`;`body` 不设底色与前景色。

---

## 6. 禁止模式

| 禁止 | 正确做法 |
|------|----------|
| 组件里写原语色(`bg-zinc-*` / `text-amber-*` / `fill-amber-400`) | 语义 token(`bg-muted` / `text-warning` / `fill-warning`) |
| `text-[Npx]` 任意值 | 用刻度类;缺档在 `@theme` 加 `--text-*` |
| `:root` 写字面颜色(`#fff` / `oklch()` / `rgb()` / `hsl()`,含阴影里的) | 按 §2「颜色怎么写」:内置 `var(--color-*)` + `--alpha()` + 需要时 `light-dark()` |
| `dark:` 变体或 `.dark` 双份维护 | `light-dark()` + 面板根 `scheme-*` |
| `@theme inline` 里写具体颜色值 | 只写 `var(--background)` 这类映射 / `calc(var(--radius) ± N)` |
| 新建 `tailwind.config.js` | 主题只活在 `src/index.css` |
| 把 `muted`(底)当成字色、`accent` 当成底座 | 见 §2 角色表 |
| 同一元素 `bg-accent text-foreground`(以及 `destructive` / `warning` 同错) | `bg-accent text-accent-foreground` |
| 因「成对」把徽章写成 `bg-muted text-muted-foreground` | `bg-muted text-foreground`;`muted-foreground` 只用于本来就要淡的字 |
| 在 `body` / `html` 上设 `background` / `color` 或把 `color-scheme` 挂到 `html` | 面板根承担配色与 `scheme-*` |
