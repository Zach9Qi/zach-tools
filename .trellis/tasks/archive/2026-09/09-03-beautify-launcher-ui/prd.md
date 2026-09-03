# 美化启动器 UI（MacOS 原生质感风格）

## Goal

遵循三层设计令牌与纯样式修改原则，在不改动任何功能逻辑的前提下，优化启动器与剪贴板历史界面的视觉层次、质感与微交互，呈现温润、精致、具有 macOS 原生质感的现代桌面 UI。

## Requirements

1. **严格遵循既有架构与规范约束**：
   - 纯样式与 UI 结构修饰调整，绝不改动任何状态管理、快捷键绑定、IPC 交互及生命周期逻辑。
   - 遵循三层设计令牌架构（`:root` -> `@theme inline` -> 组件语义工具类）：
     - 组件中禁止使用硬编码颜色（如 `zinc-*`, `amber-*` 等原始色）或任意数值字号 `text-[Npx]`。
     - 若需要微调基准色（深/浅色背景质感、柔和高光/边框透明度、焦点环等），必须且仅在 `src/index.css` 的 `:root` 统一调整，保持 `light-dark()` 双色支持。
     - 禁止使用外壳投影（因无窗口透明 padding），启动器边框保持 `border border-border`。

2. **启动器主面板与全局风格（MacOS 原生质感）**：
   - **面板整体与色调**：浅色模式下更温润通透，深色模式下更有层次（避免生硬死黑或高反差冷灰），微调背景与边框的质感。
   - **搜索栏区域**：
     - 主页搜索栏（`HomeSearchBar.vue`）与工具搜索栏（`ToolSearchBar.vue`）：优化输入框字号、字重与占位符质感，统一搜索图标与快捷键标签的微对齐。
     - 快捷键键帽（`KeyboardKey.vue`）：打磨按键胶囊质感（精细边框、微立体背景感、更清爽小巧的字体呈现）。
     - 工具退出徽章（`ToolSearchBar.vue`）：增强可点击胶囊感与 hover 微过渡。
   - **主页内容与磁贴网格（`ResultsPanel.vue`, `ToolSection.vue`, `ToolTile.vue`）**：
     - 分区标题（`ToolSection.vue`）：标题增加微字距、清晰层级；优化“全部 >”指示样式。
     - 磁贴卡片（`ToolTile.vue`）：告别扁平单调！图标底座增加温和微边框与轻微背景层级，hover / 键盘选中时具备柔和平滑的背景过渡（`transition-colors`），图标与文字排版更舒展和谐。
     - 空状态（`ResultsPanel.vue`）：打磨图标底座与空状态文案的排版层次。
   - **状态栏 / 页脚（`LauncherFooter.vue`）**：
     - 底部按键提示栏增加上边框微弱分隔感或背景细微对比，字体更加精致，快捷键与提示文案对齐更工整。

3. **剪贴板工具页精致化（`ClipboardPage.vue` 等）**：
   - **分类与筛选标签（`ClipboardFilterTabs.vue`）**：
     - 优化 macOS 风格分段胶囊效果（Segmented Control 质感），选中有精致底色与清晰文字，收藏按钮带柔和点缀。
   - **条目列表项（`ClipboardListItem.vue`）**：
     - 列表项高度、内边距与圆角微调，hover 与 selected 状态拥有更自然的过度。
     - 图标底座增加圆角与精致背景，条目单行截断文本排版更清晰，相对时间更淡雅。
   - **条目详情预览（`ClipboardDetailPane.vue`）**：
     - 右侧详情头部工具栏：按钮尺寸与 hover 反馈优化，收藏、复制、删除操作图标更精致。
     - 预览正文：行高、内边距与字体优化（使用更加舒适阅读的排版），长文本截断提示更含蓄。

4. **过渡与微动效**：
   - 适当添加 `transition-colors duration-150` 或 `transition-all` 等极轻量微过渡，使 hover / 选中状态切换更丝滑自然，摆脱生硬的即时跳变。

## Acceptance Criteria

- [x] 零功能性退化：搜索、导航、快捷键（上下左右、Enter、Esc、Alt+Enter、Ctrl+F）、剪贴板复制/粘贴/删除/收藏全部正常工作。
- [x] 规范合规：组件模板中无硬编码字面颜色、无 `zinc-*` / `amber-*` 类，所有颜色均经由语义 token 消费。
- [x] 浅色 / 深色模式双端视觉正常：在 macOS 原生质感下，浅色温和精致，深色深邃有层级。
- [x] 代码质量与门禁全部通过：`bun run format && bun run lint && bun run test && bun run build` 零警告零错误。
