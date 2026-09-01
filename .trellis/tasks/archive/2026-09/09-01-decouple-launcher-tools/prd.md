# 前端解耦:launcher 与 tools 依赖方向单向化

## 背景

当前 `src/launcher/`(外壳)与 `src/tools/`(工具)互相依赖:

- launcher → tools:经 `tools/registry.ts` 查表加载工具(设计内,保留)
- tools → launcher:`tools/clipboard/composables/useClipboardPage.ts` 深入导入了外壳内部实现
  - `@/launcher/composables/useKeymap`
  - `@/launcher/lib/window`(`onLauncherOpen`)

双向依赖使层级成环:工具增多后,外壳内部重构会被每个工具牵制。对照社区单向依赖原则(FSD),用户决策采用**彻底分离**方案:共用设施上提到根级共享层,而非 launcher 暴露 public API。

## 目标

依赖方向固定为单向:`launcher → tools → 共享层(src/lib、src/composables)`,`src/tools/**` 不出现任何 `@/launcher` 导入。

## 改动范围

1. **文件移动(内容不变)**
   - `src/launcher/composables/useKeymap.ts` → `src/composables/useKeymap.ts`(外壳与工具页共用的快捷键登记处)
   - `src/launcher/lib/window.ts` → `src/lib/window.ts`(启动器窗口 IPC 封装:hide / resize / open / close 事件,外壳与工具都要消费)
2. **导入路径更新(共 6 处)**
   - useKeymap 消费方:`useClipboardPage.ts`、`LauncherFooter.vue`、`useRowNavigation.ts`
   - window 消费方:`useClipboardPage.ts`、`useLauncher.ts`、`useAutoHeight.ts`
3. **spec 同步**
   - `frontend/directory-structure.md`:目录树、根级共享层说明、新增「依赖方向」规则(tools 禁止 import `@/launcher/*`)
   - `frontend/composable-guidelines.md`、`frontend/quality-guidelines.md`、`frontend/index.md`:路径引用与禁止模式清单更新
   - `tauri/module-organization.md`、`tauri/commands-and-ipc.md`:对前端封装文件的路径引用更新

## 明确不做

- 不引入 ESLint 边界强制(独立任务)
- 不重命名事件与函数(`onLauncherOpen` 等保持与后端事件名对应)
- 不动 `useToolView`(仅 launcher 内部与外壳消费,无工具依赖)

## 验收标准

- [x] `src/tools/**` 中 grep `@/launcher` 零命中
- [x] 旧路径文件已删除,全仓库无对旧路径的引用残留
- [x] `bun run build`(vue-tsc + vite)通过
- [x] spec 中所有受影响路径已更新,并新增依赖方向规则
- [x] `useKeymap` 模块级共享状态仍是单实例(全部消费方指向同一新路径,旧文件已删,vue-tsc 保证无双实例)
