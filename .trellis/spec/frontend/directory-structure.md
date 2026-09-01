# 目录结构

> 前端按功能模块组织,不按文件类型在根上摊平。判断一个文件放哪,先问「它属于哪个功能」。

---

## 顶层布局(真实结构)

```
src/
├── App.vue              # 只做布局与顶层挂载,不含业务(全文 9 行)
├── main.ts              # createApp 入口
├── index.css            # Tailwind 入口 + @theme 语义色板(全局唯一样式文件)
├── composables/         # 跨模块共享 composable:useKeymap(快捷键登记,外壳与工具页共用)
├── lib/                 # 跨模块共享普通代码:runtime.ts(isTauriRuntime)、window.ts(启动器窗口 IPC 封装)
├── launcher/            # 启动器外壳与主页
│   ├── components/      # LauncherPanel / ResultsPanel / SearchInput / ToolTile ...
│   └── composables/     # useLauncher / useResults / useToolView / useRowNavigation ...
└── tools/               # 工具体系
    ├── types.ts         # ToolItem / ToolModule 等共享契约
    ├── registry.ts      # 工具注册表(catalog + moduleOf)
    ├── match.ts         # accepts 可复用的输入形态判断(isUrl / isPath)
    ├── icons.ts         # icon 字符串 → lucide 组件映射
    └── clipboard/       # 每个工具一个目录
        ├── index.ts     # 导出 ToolModule(目录项 + 页面/动作)
        ├── components/  # ClipboardPage / ClipboardListItem ...
        ├── composables/ # useClipboardPage
        └── lib/         # api.ts(IPC 封装)/ tabs.ts / time.ts
```

---

## 功能模块的标准解剖

每个功能模块(launcher、tools/<id>)内部固定三类子目录,职责不混:

| 子目录 | 放什么 | 参考 |
|--------|--------|------|
| `components/` | SFC 组件,只做渲染绑定 | `src/tools/clipboard/components/ClipboardPage.vue` |
| `composables/` | 有响应式状态或生命周期的 `useXxx` | `src/tools/clipboard/composables/useClipboardPage.ts` |
| `lib/` | 无响应式的普通函数:IPC 封装、纯逻辑、常量表 | `src/tools/clipboard/lib/api.ts`、`time.ts`、`tabs.ts` |

**IPC 封装必须落在 `lib/`,归属看消费方**:单模块私有的放模块内(剪贴板 → `tools/clipboard/lib/api.ts`);外壳与工具都要消费的放根级共享层(启动器窗口 → `src/lib/window.ts`)。组件与 composable 里不出现字符串命令名 / 事件名。

**根级共享层(`src/lib/`、`src/composables/`)不是默认堆放处**:进入门槛是「≥2 个功能模块真实消费」。现有居民:`src/lib/runtime.ts`、`src/lib/window.ts`、`src/composables/useKeymap.ts`。单模块私有的照旧放模块内;`src/components/` 目前不存在,没有真实需要就不要创建。

---

## 依赖方向(硬规则)

只有一个方向:**launcher → tools → 共享层(`src/lib/`、`src/composables/`)**。

- launcher 可 import `@/tools/*`(仅经 `registry` 查表消费)与共享层
- **tools 禁止 import `@/launcher/*`**:工具需要的外壳能力(快捷键登记 `useKeymap`、窗口事件 `onLauncherOpen` 等)一律来自共享层
- 共享层不 import launcher 与 tools
- 新增「外壳与工具都要用」的能力时,直接落在共享层,不要让 tools 反向伸进外壳内部
- 该规则已由 oxlint 强制:`.oxlintrc.json` 对 `src/tools/**` 的 overrides 配置了 `no-restricted-imports`,违规导入在 `bun run lint` 直接报错

> 由来:useKeymap 与 window.ts 曾在 launcher 内部,clipboard 反向深导入形成层级环,任务 `09-01-decouple-launcher-tools` 将其上提修复,此后保持单向。

---

## 新增一个工具的完整路径

以 `src/tools/clipboard/` 为模板,四步接入,外壳零改动:

1. 建 `src/tools/<id>/`,在 `index.ts` 导出一个 `ToolModule`(view 型带 `page`,launch 型带 `run`,见 `src/tools/types.ts` 的判别联合)
2. 图标在 `src/tools/icons.ts` 的映射表登记一行(lucide 名 → 组件)
3. 在 `src/tools/registry.ts` 的 `modules` 数组加入该模块
4. 目录项的 `accepts` 需要输入形态判断时,复用/扩充 `src/tools/match.ts`

外壳(`LauncherPanel`)只认 registry 的查表结果,**禁止**出现 `if (activeView === "clipboard")` 这类按工具 id 分支的代码。

---

## 命名与导入

- 组件文件名 = 组件名,多单词 PascalCase(`SearchBar.vue` 而非 `Bar.vue`);工具模块内组件加工具前缀(`ClipboardListItem.vue`)避免全局重名
- composable 文件名 = 导出的函数名(`useResults.ts` 导出 `useResults`),一个文件只导出一个 composable
- 导入一律走 `@/` 别名(指向 `src/`),不写 `../../` 相对路径
- 模块的对外入口是 `index.ts`(如 `@/tools/clipboard`),内部文件之间可以直接互相引用完整路径
