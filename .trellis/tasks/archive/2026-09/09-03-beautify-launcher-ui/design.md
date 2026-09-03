# 启动器 UI 美化设计方案（MacOS 原生质感风格）

## 1. 设计理念与风格基调

macOS 原生设计风格的精髓在于：
- **温和与通透的层级**：不依靠厚重的硬边框和强烈的高反差，而是依靠微妙的背景明度阶梯（Background -> Muted -> Accent）和细致的内描边（Subtle Border）。
- **精致紧凑的胶囊与控件**：按键、徽章、Tab 具有圆润且不失严谨的胶囊形态，分段选择器（Segmented Control）具有整体卡片底托感。
- **微动效与顺滑交互**：状态切换带有轻量的颜色渐变过渡（150ms 缓动），避免机械式生硬跳变。
- **排版呼吸感**：清晰的字重对比（Medium/Semibold 标题，常规正文，精巧弱化的时间与元信息）。

---

## 2. 颜色与设计令牌微调 (`src/index.css`)

严格遵守三层架构：只在 `:root` 调整原始变量，`@theme inline` 与组件消费方式不变。

### 2.1 原始颜色优化
1. **背景色 (`--background`)**：
   - 浅色：目前是纯白 `var(--color-white)`，保持纯净；
   - 深色：目前是 `var(--color-zinc-900)`（偏死黑硬灰），调整为带有一点质感梯度的深色体系，或保持 `zinc-900` 但让次级底形成柔和阶梯。
2. **次级底 (`--muted`)**：
   - 浅色：由 `zinc-100` 改为稍微柔润的浅底（如 `zinc-100` 或 `--alpha(var(--color-zinc-200) / 50%)`），使图标底座更具轻盈感；
   - 深色：由 `zinc-800` 优化为 `var(--color-zinc-800)` / 柔和微底。
3. **交互选中态 (`--accent`)**：
   - 浅色：目前初值与 muted 相同为 `zinc-100`。在 macOS 风格中，选中态可稍加鲜明一点，如 `--alpha(var(--color-zinc-900) / 6%)`（深浅模式下更具融合感）或微调。
   - 深色：`--alpha(var(--color-white) / 10%)`，让选中与 hover 呈现轻盈的高亮层。
4. **描边 (`--border` / `--input`)**：
   - 浅色 border：更细腻柔和 `--alpha(var(--color-zinc-300) / 40%)`，告别硬线条；
   - 控件描边 input：`--alpha(var(--color-zinc-400) / 30%)`，保证键帽和控件有细腻的微轮廓。

---

## 3. 组件视觉层改造细节

### 3.1 快捷键胶囊 (`KeyboardKey.vue`)
- **现状**：单纯的 `h-4 min-w-4 rounded-sm border border-input bg-muted px-1`，比较生硬。
- **优化**：
  - 增加微阴影/双层质感（通过 `shadow-2xs` 或精细边框，符合 Mac 键盘键帽感）。
  - 微调高度为 `h-4.5 min-w-4.5`，字形居中对齐，略微加粗字体或使用更紧凑等宽质感。

### 3.2 搜索栏 (`HomeSearchBar.vue` / `ToolSearchBar.vue`)
- **主页搜索栏**：
  - 放大器图标加入淡淡的品牌感或柔和的焦点呼应；
  - 搜索框文字排版：主页占位符与输入字体优化，光标流动更舒适；
  - 快捷键提示排列更加小巧雅致。
- **工具搜索栏**：
  - 工具徽章（`ToolSearchBar.vue`）：由简单的 `bg-muted` 提升为类似 macOS 导航栏返回胶囊：带 `border border-border/50`、hover 时 `hover:bg-accent`、微动效 `transition-colors`、微图标对齐。

### 3.3 主页磁贴与列表 (`ToolSection.vue`, `ToolTile.vue`, `ResultsPanel.vue`)
- **分区标题 (`ToolSection.vue`)**：
  - 标题增加稍微紧凑的大写字母间距（tracking）或字重（`font-semibold text-2xs` 或 `text-xs font-medium`），视觉层级分明。
  - “全部 >”按钮增加 hover 色彩流动。
- **磁贴卡片 (`ToolTile.vue`)**：
  - 目前仅是一个普通 button + 方框。
  - 改造：
    - 外层卡片：增加平滑过渡 `transition-all duration-150 ease-out`。
    - 选中态：不仅有 `bg-accent`，同时轻微的卡片轮廓感。
    - 图标底座：采用略带层次的 `rounded-xl bg-muted border border-border/40`，让图标更有独立 app icon 的桌面质感。
    - 标题文本：两行限制下，行高更紧凑精致，hover/selected 时主文字更清晰。

### 3.4 底部状态栏 (`LauncherFooter.vue`)
- **现状**：`flex h-10 shrink-0 items-center justify-end px-4`，略显松散。
- **优化**：
  - 增加顶部微细分割线 `border-t border-border/40` 或极淡背景对比；
  - 快捷键提示条目间增加精致间距与柔和的分隔感；
  - 状态文本更加细致（如 `text-muted-foreground/80`）。

### 3.5 剪贴板历史页面 (`ClipboardPage.vue` 等)
- **分类标签栏 (`ClipboardFilterTabs.vue`)**：
  - 打造标准的 macOS 胶囊分段器（Segmented Control）：背景底壳包裹，选中的 tab 呈现高亮卡片效果（`bg-accent shadow-xs` 等），并有平滑切换过渡。
  - 收藏过滤按钮作为独立胶囊，点亮时有温和的金色点缀。
- **列表条目 (`ClipboardListItem.vue`)**：
  - 条目高度与左右 padding 微调，图标底座微边框化；
  - 文本预览首行样式优化；
  - 选中项（`selected`）增加高对比度与更柔和边缘，过渡动画 `transition-colors duration-100`。
- **右栏详情 (`ClipboardDetailPane.vue`)**：
  - 顶部操作栏：按钮从单纯的图标改为小巧的微交互按钮（hover 时圆润底座、激活时触感反馈）。
  - 内容文本区：设置更适合阅读的行高（如 `leading-relaxed`），保持等宽/无衬线字体的清晰度，空态与截断提示更优雅。

---

## 4. 兼容性与约束检查

- **纯 CSS/Tailwind 调整**：不增加任何额外的重型运行时库。
- **类型系统安全**：无 TS 破坏性改动。
- **CI 门禁与命令**：`bun run lint`、`bun run test`、`bun run build`、`bun run format` 必须 100% 绿色。
