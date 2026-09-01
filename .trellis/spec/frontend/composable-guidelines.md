# 组合式函数规范

> 本项目用 Vue 组合式函数(composable)承载全部有状态逻辑;没有 Pinia、没有类。
> 命名 `useXxx`,一个文件只导出一个 composable,文件名与函数名一致。

---

## 归属与分层

- 单模块私有的 composable 跟功能走:`useLauncher` / `useResults` 在 `src/launcher/composables/`,`useClipboardPage` 在 `src/tools/clipboard/composables/`;外壳与工具共用的放根级 `src/composables/`(如 `useKeymap`),归属规则见[目录结构](./directory-structure.md)的依赖方向一节
- **composable 不直接 `invoke` / `listen`**,只调用 `lib/` 的封装函数(`useClipboardPage` → `tools/clipboard/lib/api.ts` 与 `src/lib/window.ts`)
- 无响应式、无生命周期的纯逻辑(时间格式化、tab 定义、输入判断)是 `lib/` 里的普通函数,不要包成 composable(`tools/clipboard/lib/time.ts`、`tabs.ts`)

---

## 签名约定

- 响应式输入声明为 `Ref`,调用方用 `toRef(props, "query")` 传入,保持响应链(`useClipboardPage(query: Ref<string>)`、`ClipboardPage.vue` 的调用处)
- 参数超过一个或含回调时,收拢成 options 接口,导出并给每个字段写文档注释:

```ts
/** useResults 配置 */
export interface UseResultsOptions {
  /** 当前搜索词(来自外壳 useLauncher) */
  query: Ref<string>;
  /** 回车 / 点击磁贴时回调。source 为条目所在分区 */
  onActivate: (tool: ToolItem, source: SectionKey) => void;
}
```

- 返回值是可解构的对象:响应式状态(ref / computed)+ 动作函数;不返回类实例
- 可复用逻辑做成泛型时只抽象「形状」,业务语义由调用方注入(`useRowNavigation<T>` 只认「按行分组的可选项」,不关心条目是磁贴还是列表行)

---

## 模块级共享状态

跨组件共享的状态写成**模块级 `ref` + 无参 composable**,所有调用方拿到同一份:

```ts
// src/launcher/composables/useToolView.ts
/** 当前打开的工具页注册单元;null = 主页。模块级状态,所有组件共享同一份 */
const activeModule = ref<ViewToolModule | null>(null);
```

适用判断:多个不相邻组件要读同一份状态(工具页态被 `LauncherPanel` / `ToolSearchBar` / `LauncherFooter` 同时消费)。只有单一使用方时用普通局部 ref,不要预先全局化。

---

## 生命周期与清理

- 事件监听在 `onMounted` 注册、`onUnmounted` 全部清理;多个 Tauri 监听用 `unlisteners` 数组 + `Promise.all` 收集,卸载时逐个调用(`useLauncher.ts`、`useClipboardPage.ts`)
- 定时器句柄存变量,卸载时 `clearTimeout`(`useClipboardPage` 的 `refreshTimer` / `copiedTimer`)
- 共享登记类状态在卸载时只清除仍属于自己的那份——页面切换时新页可能已登记(`useKeymap.ts` 卸载钩子里 `registered.value === bindings` 的比对)

---

## 快捷键:一处定义,两处派生

页面快捷键统一经 `useKeymap`(`src/composables/useKeymap.ts`,共享层,外壳与工具页都可用)登记:一条 `KeyBinding` 同时驱动 keydown 分发与页脚键帽提示,结构上杜绝「按键行为与页脚文案漂移」。

- 页面级 composable 传 bindings 登记(`useClipboardPage`、`useRowNavigation`);`LauncherFooter` 无参调用只读提示
- Esc 归外壳(`useLauncher`)分层处理,不在页面登记
- 输入法组词(`event.isComposing`)、Alt / Meta / Shift 组合一律放行;Ctrl 组合仅在绑定显式声明 `ctrl: true` 时接管——新增绑定时不要破坏这些放行规则

---

## 异步请求编排(参考实现:useClipboardPage)

`src/tools/clipboard/composables/useClipboardPage.ts` 是本项目请求编排的基准样板,新的列表类页面按同样机制处理:

- **过期丢弃**:模块内维护 `generation` 版本号,整表重拉时 `++generation`;响应回来先比版本,对不上直接丢弃,防止旧请求覆盖新结果
- **统一请求入口**:loading、try/catch、过期判断收在一个 `requestList` 函数,调用方只声明差异(拉哪页、结果如何合入)
- **防抖**:连续击键 / 事件风暴用 `setTimeout` 防抖(120ms 级),过滤维度切换则立即重拉并作废在途请求
- **keyset 分页**:游标是值锚点 `(lastUsedAt, id)`,由 `lib/api.ts` 的 `ClipboardListCursor` 承载;不用 offset 分页
- **事件合并**:后端推送按 id 去重(已存在则上浮,不重复插入);带过滤词时不在前端复刻后端匹配语义,只把事件当失效信号触发重查
- **本地镜像**:自己发起、后端不回推的变更(粘贴/复制刷新 `lastUsedAt`)在本地同步执行(`mirrorTouch`),保证下次比对状态一致

---

## 常见错误

- 在组件 script 里写防抖 / 请求 / 事件合并——移入 composable
- 忘记清理 Tauri `listen` 返回的 unlisten——窗口常驻、组件反复挂载,泄漏会累积
- 新增快捷键绕过 `useKeymap` 直接 `addEventListener`——页脚提示会漂移
- 列表选中用数组下标持久保存——插入 / 重排后漂移,应存 id 再派生下标(`selectedId` → `selectedIndex`)
