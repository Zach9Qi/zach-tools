import { computed, onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useKeymap } from "@/composables/useKeymap";
import { onLauncherOpen } from "@/lib/window";
import {
  copyClipboardItem,
  deleteClipboardItem,
  listClipboardItems,
  onClipboardNewItem,
  pasteClipboardItem,
  setClipboardFavorite,
  type ClipboardItem,
  type ClipboardListCursor,
} from "@/tools/clipboard/lib/api";
import { KIND_TABS, type ClipboardKindTab } from "@/tools/clipboard/lib/tabs";

/** 单页拉取条数,与后端默认值一致 */
const PAGE_SIZE = 50;
/** 列表重查的防抖间隔(毫秒):本地 SQLite 查询很快,只挡连续击键与事件风暴 */
const REFRESH_DEBOUNCE_MS = 120;

/**
 * 剪贴板页状态编排:keyset 分页列表、三维正交过滤(搜索词 × 类型 tab × 只看收藏)、
 * 键盘选中、粘贴/删除/收藏、新条目实时合并。
 * 生命周期与页面组件一致(退出工具页即卸载,重进重拉)。
 * 窗口隐藏期间组件仍存活,新条目靠事件持续合入本地列表;
 * 再次唤起只比对首条 id:对得上就不重置,滚动位置与选中项留着。
 */
export function useClipboardPage(query: Ref<string>) {
  /** 当前列表,与后端 (last_used_at DESC, id DESC) 全序一致 */
  const items = ref<ClipboardItem[]>([]);
  /** 选中条目 id:列表插入/重排时选中跟随条目本身,不随下标漂移 */
  const selectedId = ref<number | null>(null);
  /** 当前类型 tab 的 key,与过滤词一样作用于整个列表查询 */
  const activeKindKey = ref(KIND_TABS[0]?.key ?? "all");
  /** 「只看收藏」开关,与类型 tab、搜索词正交叠加 */
  const favoriteOnly = ref(false);
  /** 是否正在请求(重查期间挡住 loadMore) */
  const loading = ref(false);
  /** 后端已无更多数据 */
  const exhausted = ref(false);
  /** 列表重置(过滤词变化 / 切分类 / 藏窗口时漏了事件)完成计数,页面组件据此把滚动条拉回顶部 */
  const refreshTick = ref(0);
  /** 最近一次「仅复制」成功的条目 id,短暂保留用于按钮反馈 */
  const copiedId = ref<number | null>(null);

  /** 选中条目下标;选中项不在列表中(如刚被删)时为 -1 */
  const selectedIndex = computed(() =>
    items.value.findIndex((item) => item.id === selectedId.value),
  );
  /** 当前选中条目,详情栏据此渲染;列表为空时为 null */
  const selected = computed(() => items.value[selectedIndex.value] ?? null);
  /** 当前类型 tab 的过滤定义;注册表异常时兜底为不限类型 */
  const activeKind = computed<ClipboardKindTab>(
    () => KIND_TABS.find((tab) => tab.key === activeKindKey.value) ?? { key: "all", label: "全部" },
  );

  /**
   * 当前列表版本号。整表重拉(进页、改搜索词、再次唤起)时加一,翻页不加。
   * 请求发出时记下当时的版本,回来时对不上就丢掉,避免旧搜索或旧翻页盖住新结果。
   */
  let generation = 0;

  function trimmedQuery(): string | undefined {
    const keyword = query.value.trim();
    return keyword ? keyword : undefined;
  }

  /** 把条目放到列表顶部(已在列表中则先移除旧位置),镜像后端的 last_used_at 排序 */
  function placeOnTop(item: ClipboardItem) {
    const index = items.value.findIndex((it) => it.id === item.id);
    if (index !== -1) {
      items.value.splice(index, 1);
    }
    items.value.unshift(item);
  }

  /** 用首屏结果整体重置:回到顶部、选中第一项 */
  function applyReset(result: ClipboardItem[]) {
    items.value = result;
    exhausted.value = result.length < PAGE_SIZE;
    selectedId.value = result[0]?.id ?? null;
    refreshTick.value += 1;
  }

  /**
   * 列表请求的统一入口:loading、错误、过期丢弃都在这里处理。
   * 调用方只声明差异:要不要作废还在路上的请求、拉哪一页、结果如何合入。
   */
  async function requestList(options: {
    /** 为 true 时版本号加一,还在路上的请求回来后作废(不写列表、不关 loading) */
    invalidate?: boolean;
    /** keyset 翻页游标,缺省拉首屏 */
    cursor?: ClipboardListCursor;
    /** 把本次结果合入列表;发出后又整表重拉过则不调用 */
    apply: (result: ClipboardItem[]) => void;
  }): Promise<void> {
    // 记下出发时的版本;invalidate 则先换新版本,让旧请求对不上号
    const gen = options.invalidate ? ++generation : generation;
    loading.value = true;
    try {
      const result = await listClipboardItems({
        query: trimmedQuery(),
        kind: activeKind.value.kind,
        favoriteOnly: favoriteOnly.value || undefined,
        limit: PAGE_SIZE,
        cursor: options.cursor,
      });
      if (gen === generation) {
        options.apply(result);
      }
    } catch (error) {
      console.error("剪贴板列表请求失败:", error);
    } finally {
      if (gen === generation) {
        loading.value = false;
      }
    }
  }

  /**
   * 拉取首屏。mode 为 reset 时无条件重置(挂载 / 过滤词变化);
   * 为 sync 时只比首条 id:对不上说明藏窗口时漏了事件,才整表重拉;
   * 对得上就不重置,滚动和选中都留着。
   */
  async function refreshList(mode: "reset" | "sync") {
    await requestList({
      invalidate: true,
      apply: (result) => {
        if (mode === "reset" || result[0]?.id !== items.value[0]?.id) {
          applyReset(result);
        }
      },
    });
  }

  /**
   * 滚动到底时拉取下一页。游标锚定在末行的 (lastUsedAt, id) 值上,
   * 期间的置顶/删除不会造成跳行或重行,无需任何位置记账。
   */
  async function loadMore() {
    const last = items.value[items.value.length - 1];
    if (loading.value || exhausted.value || !last) {
      return;
    }
    await requestList({
      cursor: { lastUsedAt: last.lastUsedAt, id: last.id },
      apply: (result) => {
        exhausted.value = result.length < PAGE_SIZE;
        // 在途期间事件可能已把本页某条目置顶(重复复制),响应是移动前的快照,按 id 去重
        const known = new Set(items.value.map((item) => item.id));
        items.value.push(...result.filter((item) => !known.has(item.id)));
      },
    });
  }

  /**
   * 本地镜像后端的 touch:粘贴/复制会刷新库里的 last_used_at 并把条目排到最前,
   * 而自写事件被后端抑制不会推回来,这里同步置顶,让本地第一条跟库里一致,
   * 下次唤起比对首条时才不会误以为漏了事件。
   */
  function mirrorTouch(item: ClipboardItem) {
    placeOnTop({ ...item, lastUsedAt: Date.now() });
  }

  /** 粘贴条目;后端负责隐藏窗口、还原焦点并注入 Ctrl+V */
  async function paste(item: ClipboardItem) {
    try {
      await pasteClipboardItem(item.id);
    } catch (error) {
      console.error("粘贴失败:", error);
      return;
    }
    mirrorTouch(item);
  }

  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  /** 仅复制到系统剪贴板(面板保持打开),成功后短暂点亮反馈 */
  async function copy(item: ClipboardItem) {
    try {
      await copyClipboardItem(item.id);
    } catch (error) {
      console.error("复制失败:", error);
      return;
    }
    mirrorTouch(item);
    copiedId.value = item.id;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copiedId.value = null;
    }, 1000);
  }

  /** 从列表移除下标处条目;移除的是选中项时就近改选同位置的下一条 */
  function dropFromList(index: number) {
    const [removed] = items.value.splice(index, 1);
    if (removed && selectedId.value === removed.id) {
      const fallback = items.value[Math.min(index, items.value.length - 1)];
      selectedId.value = fallback ? fallback.id : null;
    }
  }

  /** 删除条目 */
  async function remove(item: ClipboardItem) {
    try {
      await deleteClipboardItem(item.id);
    } catch (error) {
      console.error("删除失败:", error);
      return;
    }
    const index = items.value.findIndex((it) => it.id === item.id);
    if (index !== -1) {
      dropFromList(index);
    }
  }

  /**
   * 切换条目收藏状态,成功后本地同步(不重拉):
   * 一般视图原地更新星标;收藏视图里取消收藏意味着条目离开当前列表,按删除的方式移除。
   */
  async function toggleFavorite(item: ClipboardItem) {
    const favorite = !item.isFavorite;
    try {
      await setClipboardFavorite(item.id, favorite);
    } catch (error) {
      console.error("收藏操作失败:", error);
      return;
    }
    const index = items.value.findIndex((it) => it.id === item.id);
    if (index === -1) {
      return;
    }
    if (favoriteOnly.value && !favorite) {
      dropFromList(index);
      return;
    }
    items.value[index] = { ...item, isFavorite: favorite };
  }

  /** 条目是否属于当前过滤视图(类型与收藏两个维度;关键字语义留给后端) */
  function matchesActiveFilters(item: ClipboardItem): boolean {
    if (activeKind.value.kind && item.kind !== activeKind.value.kind) {
      return false;
    }
    if (favoriteOnly.value && !item.isFavorite) {
      return false;
    }
    return true;
  }

  /**
   * 新条目落库(外部复制):不属于当前过滤视图的直接忽略(切换过滤时会整表重拉);
   * 无过滤词时直接置顶(重复复制则上浮);
   * 有过滤词时不在前端复刻后端的匹配语义,只当失效信号,防抖后整表重查
   */
  function handleNewItem(item: ClipboardItem) {
    if (!matchesActiveFilters(item)) {
      return;
    }
    if (trimmedQuery()) {
      scheduleRefresh();
      return;
    }
    placeOnTop(item);
  }

  /** 鼠标悬停等场景直接指定选中项 */
  function select(index: number) {
    selectedId.value = items.value[index]?.id ?? null;
  }

  /** ↑↓ 首尾回绕移动选中(与主页磁贴导航一致);选中项刚被删(-1)时从边界重新开始 */
  function moveSelection(delta: 1 | -1) {
    const count = items.value.length;
    if (count === 0) {
      return;
    }
    const current = selectedIndex.value;
    const next = current === -1 ? (delta === 1 ? 0 : count - 1) : (current + delta + count) % count;
    selectedId.value = items.value[next]?.id ?? null;
  }

  // 按键、页脚提示与回调同源登记;←→ 不绑定,留给输入框光标(过滤框改词以退格为主)。
  // Tab 轮切类型只在注册表有多个 tab 时登记(一期只有「全部」,轮切无意义则不出现在页脚)
  useKeymap([
    {
      keys: ["ArrowUp", "ArrowDown"],
      label: "选择",
      onPress: (event) => moveSelection(event.key === "ArrowDown" ? 1 : -1),
    },
    {
      keys: ["Enter"],
      label: "粘贴",
      onPress: () => {
        if (selected.value) {
          void paste(selected.value);
        }
      },
    },
    ...(KIND_TABS.length > 1
      ? [
          {
            keys: ["Tab"],
            label: "分类",
            onPress: () => cycleKind(),
          },
        ]
      : []),
    {
      keys: ["f"],
      ctrl: true,
      label: "只看收藏",
      onPress: () => toggleFavoriteFilter(),
    },
    {
      keys: ["d"],
      ctrl: true,
      label: "收藏",
      onPress: () => {
        if (selected.value) {
          void toggleFavorite(selected.value);
        }
      },
    },
    {
      keys: ["Delete"],
      label: "删除",
      onPress: () => {
        if (selected.value) {
          void remove(selected.value);
        }
      },
    },
  ]);

  // 过滤词变化与「有过滤词时的新条目事件」共用的防抖重查
  let refreshTimer: ReturnType<typeof setTimeout> | undefined;
  function scheduleRefresh() {
    clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => void refreshList("reset"), REFRESH_DEBOUNCE_MS);
  }
  watch(query, scheduleRefresh);

  /** 过滤维度(类型 / 收藏开关)变化后的统一动作:立即整表重拉,在途请求与待发防抖一并作废 */
  function refreshOnFilterChange() {
    clearTimeout(refreshTimer);
    void refreshList("reset");
  }

  /** 切换类型 tab,保留收藏开关与搜索词叠加过滤 */
  function selectKind(key: string) {
    if (activeKindKey.value === key) {
      return;
    }
    activeKindKey.value = key;
    refreshOnFilterChange();
  }

  /** Tab 键向右轮切类型(循环) */
  function cycleKind() {
    const index = KIND_TABS.findIndex((tab) => tab.key === activeKindKey.value);
    const next = KIND_TABS[(index + 1) % KIND_TABS.length];
    if (next) {
      selectKind(next.key);
    }
  }

  /** 开关「只看收藏」,与类型 tab、搜索词叠加过滤 */
  function toggleFavoriteFilter() {
    favoriteOnly.value = !favoriteOnly.value;
    refreshOnFilterChange();
  }

  let unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    // 首屏立即拉(可能带着主页传入的搜索词)
    void refreshList("reset");
    unlisteners = await Promise.all([
      onClipboardNewItem(handleNewItem),
      // 隐藏期间列表由事件维护;唤起只比对首条,对得上就不重置
      onLauncherOpen(() => void refreshList("sync")),
    ]);
  });

  onUnmounted(() => {
    clearTimeout(refreshTimer);
    clearTimeout(copiedTimer);
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });

  return {
    items,
    selectedIndex,
    selected,
    copiedId,
    loading,
    exhausted,
    refreshTick,
    kindTabs: KIND_TABS,
    activeKind,
    favoriteOnly,
    selectKind,
    toggleFavoriteFilter,
    select,
    paste,
    copy,
    remove,
    toggleFavorite,
    loadMore,
  };
}
