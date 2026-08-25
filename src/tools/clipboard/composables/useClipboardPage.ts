import { computed, onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { onLauncherOpen } from "@/launcher/lib/window";
import {
  copyClipboardItem,
  deleteClipboardItem,
  listClipboardItems,
  onClipboardNewItem,
  pasteClipboardItem,
  type ClipboardItem,
} from "@/tools/clipboard/lib/api";

/** 单页拉取条数,与后端默认值一致 */
const PAGE_SIZE = 100;
/** 输入过滤的防抖间隔(毫秒):本地 SQLite 查询很快,只挡连续击键 */
const FILTER_DEBOUNCE_MS = 120;

/**
 * 剪贴板页状态编排:分页列表、输入过滤、键盘选中、粘贴/删除、新条目实时插入。
 * 生命周期与页面组件一致(退出工具页即卸载,重进重拉);
 * 窗口隐藏期间组件仍存活(保留现场),再次唤起经 launcher-open 刷新列表。
 */
export function useClipboardPage(query: Ref<string>) {
  /** 当前列表,已按最近使用倒序 */
  const items = ref<ClipboardItem[]>([]);
  /** 键盘 / 悬停选中的下标 */
  const selectedIndex = ref(0);
  /** 是否正在请求(refresh 期间挡住 loadMore) */
  const loading = ref(false);
  /** 后端已无更多数据 */
  const exhausted = ref(false);
  /** refresh 完成计数,页面组件据此把滚动条拉回顶部 */
  const refreshTick = ref(0);
  /** 当前选中条目,详情栏据此渲染;列表为空时为 null */
  const selected = computed(() => items.value[selectedIndex.value] ?? null);
  /** 最近一次「仅复制」成功的条目 id,短暂保留用于按钮反馈 */
  const copiedId = ref<number | null>(null);

  /**
   * 已从后端分页拉走的条数。loadMore 的 offset 用它而不是 items.length:
   * 事件插入的新条目会让两者错位,删除时回退一位,避免分页跳过或重复。
   */
  let fetchedCount = 0;
  /** 请求代次:refresh 递增,旧请求的迟到响应按代次丢弃 */
  let generation = 0;

  function trimmedQuery(): string | undefined {
    const keyword = query.value.trim();
    return keyword ? keyword : undefined;
  }

  /** 重置分页并拉取首屏 */
  async function refresh() {
    const gen = ++generation;
    loading.value = true;
    try {
      const result = await listClipboardItems({ query: trimmedQuery(), limit: PAGE_SIZE });
      if (gen !== generation) {
        return;
      }
      items.value = result;
      fetchedCount = result.length;
      exhausted.value = result.length < PAGE_SIZE;
      selectedIndex.value = 0;
      refreshTick.value += 1;
    } catch (error) {
      console.error("剪贴板列表加载失败:", error);
    } finally {
      if (gen === generation) {
        loading.value = false;
      }
    }
  }

  /** 滚动到底时拉取下一页 */
  async function loadMore() {
    if (loading.value || exhausted.value) {
      return;
    }
    const gen = generation;
    loading.value = true;
    try {
      const result = await listClipboardItems({
        query: trimmedQuery(),
        limit: PAGE_SIZE,
        offset: fetchedCount,
      });
      if (gen !== generation) {
        return;
      }
      fetchedCount += result.length;
      exhausted.value = result.length < PAGE_SIZE;
      // 事件插入可能已带来其中某些条目,按 id 去重后追加
      const known = new Set(items.value.map((item) => item.id));
      items.value.push(...result.filter((item) => !known.has(item.id)));
    } catch (error) {
      console.error("剪贴板列表加载更多失败:", error);
    } finally {
      if (gen === generation) {
        loading.value = false;
      }
    }
  }

  /** 粘贴条目;后端负责隐藏窗口、还原焦点并注入 Ctrl+V */
  async function paste(item: ClipboardItem) {
    try {
      await pasteClipboardItem(item.id);
    } catch (error) {
      console.error("粘贴失败:", error);
    }
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
    copiedId.value = item.id;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copiedId.value = null;
    }, 1000);
  }

  /** 删除条目并同步本地列表与分页游标 */
  async function remove(item: ClipboardItem) {
    try {
      await deleteClipboardItem(item.id);
    } catch (error) {
      console.error("删除失败:", error);
      return;
    }
    const index = items.value.findIndex((it) => it.id === item.id);
    if (index === -1) {
      return;
    }
    items.value.splice(index, 1);
    fetchedCount = Math.max(0, fetchedCount - 1);
    if (selectedIndex.value >= items.value.length) {
      selectedIndex.value = Math.max(0, items.value.length - 1);
    }
  }

  /** 新条目落库:命中当前过滤词才展示;已在列表中(重复复制)则上浮到顶 */
  function handleNewItem(item: ClipboardItem) {
    const keyword = trimmedQuery();
    // 匹配语义与后端 LIKE 对齐(ASCII 不区分大小写),中文无大小写问题
    if (keyword && !(item.textContent ?? "").toLowerCase().includes(keyword.toLowerCase())) {
      return;
    }
    const index = items.value.findIndex((it) => it.id === item.id);
    if (index !== -1) {
      items.value.splice(index, 1);
    } else {
      fetchedCount += 1;
    }
    items.value.unshift(item);
  }

  /** 鼠标悬停等场景直接指定选中项 */
  function select(index: number) {
    selectedIndex.value = index;
  }

  /** ↑↓ 选中(首尾回绕,与主页磁贴导航一致),Enter 粘贴选中项;←→ 留给输入框光标 */
  function handleKeydown(event: KeyboardEvent) {
    // 输入法组词与修饰键组合(如全局快捷键 Alt+Enter)不当作列表操作
    if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
      return;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      // 阻止光标在输入框内跳到行首 / 行尾
      event.preventDefault();
      const count = items.value.length;
      if (count === 0) {
        return;
      }
      const delta = event.key === "ArrowDown" ? 1 : -1;
      selectedIndex.value = (selectedIndex.value + delta + count) % count;
      return;
    }
    if (event.key === "Enter") {
      const item = items.value[selectedIndex.value];
      if (item) {
        void paste(item);
      }
    }
  }

  // 输入过滤防抖;首屏在挂载时立即拉(可能带着主页传入的搜索词)
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  watch(query, () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => void refresh(), FILTER_DEBOUNCE_MS);
  });

  let unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    window.addEventListener("keydown", handleKeydown);
    void refresh();
    unlisteners = await Promise.all([
      onClipboardNewItem(handleNewItem),
      // 窗口隐藏不卸载页面(保留现场),再次唤起时刷新以纳入隐藏期间的新记录
      onLauncherOpen(() => void refresh()),
    ]);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
    clearTimeout(debounceTimer);
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
    select,
    paste,
    copy,
    remove,
    loadMore,
  };
}
