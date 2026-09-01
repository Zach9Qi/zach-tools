# 目录结构

> 前端按功能模块组织,不按文件类型在根上摊平。判断一个文件放哪,先问「它属于哪个功能」。

---

## 顶层布局(真实结构)

```
src/
├── App.vue              # 只做布局与顶层挂载,不含业务(全文 9 行)
├── main.ts              # createApp 入口
├── index.css            # Tailwind 入口 + @theme 语义色板(全局唯一样式文件)
├── lib/                 # 跨模块共享代码;当前只有 runtime.ts(isTauriRuntime)
├── launcher/            # 启动器外壳与主页
│   ├── components/      # LauncherPanel / ResultsPanel / SearchInput / ToolTile ...
│   ├── composables/     # useLauncher / useResults / useKeymap / useRowNavigation ...
│   └── lib/             # window.ts(启动器窗口相关 IPC 封装)
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

**IPC 封装必须落在所属模块的 `lib/`**:启动器窗口 → `launcher/lib/window.ts`,剪贴板 → `tools/clipboard/lib/api.ts`。组件与 composable 里不出现字符串命令名 / 事件名。

**根级 `src/lib/`、`src/components/`、`src/composables/` 不是默认堆放处**:只放既不属于 launcher 也不属于任何工具的跨模块代码。当前只存在 `src/lib/runtime.ts`;后两个目录不存在,没有真实需要就不要创建。

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
