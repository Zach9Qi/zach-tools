# 组件规范

> 组件只负责「绑定与渲染」:状态与业务在 composable / lib,组件把它们接到模板上。

---

## SFC 结构

- 一律 `<script setup lang="ts">`,块顺序 script → template;**不写 style 块**——全仓库现无一处 style 块,主题与样式全部由 Tailwind 工具类 + `src/index.css` 承担
- 宏使用现状(全部类型式,零运行时对象声明):
  - props:`defineProps<{ ... }>()`,每个字段写文档注释(见 `ClipboardPage.vue` 的 `query` 注释,它同时是「工具页组件标准 prop 契约」的声明处)
  - emits:`defineEmits<{ activate: [tool: ToolItem, source: SectionKey] }>()`(`ResultsPanel.vue`)
  - v-model:`defineModel<string>({ required: true })`(`SearchInput.vue`)
  - 对外命令式 API:`defineExpose({ focus })`,仅限「父组件确需命令式触达」的场景,如窗口唤起时聚焦搜索框(`SearchInput.vue`)
- 模板引用用 `useTemplateRef<HTMLElement>("name")`(Vue 3.5 写法,见 `LauncherPanel.vue`、`ClipboardPage.vue`),不用裸 `ref(null)` 配 ref 属性——例外:`defineExpose` 场景下需要类型收窄时仍可用 `ref<HTMLInputElement | null>(null)`(`SearchInput.vue`)

---

## 编排组件与展示组件

两层分工在代码里是硬边界:

- **编排组件**(`LauncherPanel.vue`、`ClipboardPage.vue`):调 composable 拿状态与动作,再分发给子组件;script 里只允许「接线」代码(computed 派生展示文案、watch 滚动同步),不允许出现请求、后端调用、防抖等业务逻辑
- **展示组件**(`ToolTile.vue`、`ClipboardListItem.vue`、`KeyboardKey.vue`):props 进、emit 出,不 import 任何 composable,不持有跨渲染状态

emit 事件名描述**意图**而非实现:`activate`(激活条目)、`select`(悬停选中)、`toggle-favorite`,由父级决定语义;子组件不直接改父级状态。

---

## 拆分信号(出现即拆,不要犹豫)

- 组件超过约 150 行,或 template 嵌套超过 3~4 层 → 抽子组件
- `v-for` 循环体超过几行 → 抽成 `XxxItem.vue`(参考 `ClipboardListItem.vue`)
- template 里重复出现的结构 → 抽公共子组件(参考 `KeyboardKey.vue`)
- script 出现与渲染无关的业务逻辑(防抖、数据变换、请求编排)→ 移到该功能的 composable 或 `lib/`

当前所有组件均低于 150 行,`useClipboardPage` 400 行的业务被完整隔离在 composable 里,`ClipboardPage.vue` 只剩 120 行接线,这是应当维持的比例感。

---

## 样式(Tailwind v4)

- 主题定制只写在 `src/index.css` 的三层 token(`:root` 原始层 / `@theme inline` 语义层 / `@layer base`);**禁止创建 `tailwind.config.js`**。词表、角色与换肤见[设计令牌](./design-tokens.md)
- 颜色一律用语义 token:`bg-background` / `text-foreground` / `text-muted-foreground` / `border-border` / `bg-muted`(底座) / `bg-accent`(选中 / hover)。同一元素上 `bg-X` 配 `text-X-foreground`(如 `bg-accent text-accent-foreground`)。深浅色由 `light-dark()` 自动切换,**不写 `dark:` 变体**;生效前提是祖先有 `color-scheme`,本项目由面板根的 `scheme-light-dark` 提供(`LauncherPanel.vue`),新窗口/新根容器要记得带上
- 类名顺序:布局 → 尺寸/间距 → 排版 → 颜色/背景 → 边框/效果 → 状态变体;prettier-plugin-tailwindcss 会自动排序,格式化后以它为准
- 动态样式在**完整类名**之间条件切换,禁止拼接片段:

```vue
<!-- ✅ Tailwind 能静态扫描到 -->
<div :class="selected ? 'bg-accent' : ''" />
<!-- ❌ 拼接片段不会被生成 -->
<div :class="`bg-${color}-500`" />
```

- 布局层面的经验规则(源自 `LauncherPanel.vue` / `ClipboardPage.vue` 的注释):滚动容器链路上需要 `min-h-0`;高度上限等「单一事实」只定义在一处 CSS,不在 JS 里复刻

---

## 图标

- 只用 unplugin-icons + lucide:`import IconSearch from "~icons/lucide/search"`,作为组件 `<IconSearch class="size-4" />` 使用;不引入其他图标库
- 需要按字符串动态选图标时走 `src/tools/icons.ts` 的映射表(`iconOf`),未登记的返回拼图占位,不在数据结构里存 Vue 组件

---

## 交互细节惯例

- 可点击的非提交元素写 `<button type="button">`(`ToolTile.vue`)
- 点击不应抢走搜索框焦点的元素加 `@mousedown.prevent`(`ToolTile.vue`,启动器焦点常驻搜索框是产品级约束)
- 空态给图标 + 文案,文案按场景区分原因(`ClipboardPage.vue` 的 `emptyHint`:无历史 / 无匹配 / 收藏空三种)
- 键盘选中项滚入可视区:watch 选中下标 + `scrollIntoView({ block: "nearest" })`(`ClipboardPage.vue`)

---

## 常见错误

- 在外壳组件里按工具 id 写分支(`if (activeView === "clipboard")`)——一切经 `registry` 查表
- 在组件里直接 `invoke` / `listen`——必须经模块 `lib/` 的封装函数
- 手写 `dark:` 变体维护双色——用语义 token
- 给展示组件塞 composable——状态上提到编排组件
