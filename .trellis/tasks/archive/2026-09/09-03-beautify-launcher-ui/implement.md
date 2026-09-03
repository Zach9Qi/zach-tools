# 启动器 UI 美化实施计划

## 实施步骤

1. **第一阶段：设计令牌与全局样式打磨 (`src/index.css`)**
   - [x] 微调 `:root` 中的 `--muted`, `--accent`, `--border`, `--input`，提升深浅双色的温润感与通透度。
   - [x] 确保严格遵循规范：无字面颜色、无硬编码数值、使用 Tailwind 语义变量与 `light-dark()`。

2. **第二阶段：全局基础组件微雕**
   - [x] `KeyboardKey.vue`：升级按键键帽样式，精细化边框与胶囊质感。
   - [x] `HomeSearchBar.vue` & `SearchInput.vue`：优化搜索栏图标排版、快捷键对齐、输入框字重与占位符质感。
   - [x] `ToolSearchBar.vue`：打磨返回徽章按钮（胶囊外观、hover 交互）。
   - [x] `LauncherFooter.vue`：添加柔和顶部细线、精致化快捷键与操作提示排版。

3. **第三阶段：启动器主页磁贴与列表打磨**
   - [x] `ToolTile.vue`：改造应用磁贴图标底座（微边框、精细圆角）、卡片微交互动效 (`transition-all`)、文字排版。
   - [x] `ToolSection.vue`：优化分区标题排版、字间距与“全部 >”指示符。
   - [x] `ResultsPanel.vue`：优化空状态图标底座与文案排版。

4. **第四阶段：剪贴板工具页打磨**
   - [x] `ClipboardFilterTabs.vue`：升级为 macOS 胶囊分段选择器质感，优化收藏按钮样式。
   - [x] `ClipboardListItem.vue`：优化条目行距、图标底座、文本截断排版与选中态过渡。
   - [x] `ClipboardDetailPane.vue`：打磨头部操作按钮（hover/active 效果、微过渡）与正文阅读排版。
   - [x] `ClipboardPage.vue`：微调左右栏比例及分割线质感。

5. **第五阶段：验证与代码质量门禁**
   - [x] 运行 `bun run format` 格式化代码。
   - [x] 运行 `bun run lint` 确保 Oxlint 检查 0 错误 0 警告。
   - [x] 运行 `bun run test` 确保 Vitest 单元测试全数通过。
   - [x] 运行 `bun run build` 确保 TypeScript 类型与 Vite 构建全数通过。
   - [x] 确保纯样式与 UI 优化满足项目设计令牌与代码规范。
