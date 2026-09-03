# 调整启动器窗口尺寸为 800x600

## Goal

将启动器窗口尺寸由 720x512 调整为 800x600（4:3 比例），同步更新前端面板高度上限与 Tauri 窗口配置，提升启动器搜索与各工具页面（如剪贴板）的可视区域与使用体验。

## Requirements

1. **后端窗口尺寸配置**：
   - 将 `src-tauri/tauri.conf.json5` 中的 `width` 修改为 `800`，`height` 修改为 `600`。
2. **前端面板高度上限**：
   - 将 `src/launcher/components/LauncherPanel.vue` 中的面板最大高度 `max-h-128` 及工具页高度 `h-128`（`512px`）调整为 `max-h-150` 与 `h-150`（利用 Tailwind v4 间距体系内置的数字倍数，`150 * 0.25rem = 37.5rem = 600px`）。
3. **保持动态高度契约**：
   - 维持主页高度由内容自适应撑开（`useAutoHeight` 驱动 `resizeLauncherToContent`），工具页固定撑满 `600px`。
   - 前端视口读取 `window.innerWidth` 机制继续生效（窗口无边框，视口宽天然为 800）。

## Acceptance Criteria

- [ ] `src-tauri/tauri.conf.json5` 中窗口 `width` 为 800，`height` 为 600。
- [ ] `src/launcher/components/LauncherPanel.vue` 中高度上限从 512px (`128`) 更新为 600px（使用 `max-h-150` / `h-150`，主页最高封顶 600px，工具页固定 600px）。
- [ ] 运行类型检查与前端构建验证通过（`bun run type-check` 或 `bun run build`）。
- [ ] 运行 Rust 检查与编译验证通过（`cargo check`）。
