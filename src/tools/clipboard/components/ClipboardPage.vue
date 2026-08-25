<script setup lang="ts">
import { computed, nextTick, toRef, useTemplateRef, watch } from "vue";
import IconCheck from "~icons/lucide/check";
import IconClipboard from "~icons/lucide/clipboard";
import IconCopy from "~icons/lucide/copy";
import IconSearchX from "~icons/lucide/search-x";
import IconTrash2 from "~icons/lucide/trash-2";
import ClipboardListItem from "@/tools/clipboard/components/ClipboardListItem.vue";
import { useClipboardPage } from "@/tools/clipboard/composables/useClipboardPage";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const props = defineProps<{
  /** 搜索框当前内容,作为页内过滤词(工具页组件的标准 prop 契约) */
  query: string;
}>();

const listEl = useTemplateRef<HTMLElement>("listEl");

const {
  items,
  selectedIndex,
  selected,
  copiedId,
  loading,
  refreshTick,
  select,
  paste,
  copy,
  remove,
  loadMore,
} = useClipboardPage(toRef(props, "query"));

/** 过滤词(去空白),空态文案据此区分「无历史」与「无匹配」 */
const keyword = computed(() => props.query.trim());

/** 详情正文的渲染上限:条目文本上限 10MB,全量塞进 DOM 会卡死渲染 */
const DETAIL_MAX_CHARS = 5000;

const selectedText = computed(() => selected.value?.textContent ?? "");
const detailText = computed(() => selectedText.value.slice(0, DETAIL_MAX_CHARS));
const detailTruncated = computed(() => selectedText.value.length > DETAIL_MAX_CHARS);

/** 详情头部元信息:字符数 + 相对时间 */
const detailMeta = computed(() => {
  if (!selected.value) {
    return "";
  }
  return `文本 · ${selectedText.value.length} 字符 · ${formatRelativeTime(selected.value.lastUsedAt)}`;
});

// 键盘选中后让条目滚进可视区
watch(selectedIndex, async (index) => {
  await nextTick();
  listEl.value?.children[index]?.scrollIntoView({ block: "nearest" });
});

// 列表刷新(输入过滤 / 窗口唤起)后回到顶部
watch(refreshTick, async () => {
  await nextTick();
  listEl.value?.scrollTo({ top: 0 });
});

/** 距底不足 200px 时拉取下一页 */
function handleScroll() {
  const el = listEl.value;
  if (el && el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
    void loadMore();
  }
}
</script>

<template>
  <div class="flex min-h-0 flex-1 border-y border-line">
    <div
      v-if="items.length === 0 && !loading"
      class="flex flex-1 flex-col items-center justify-center gap-3"
    >
      <div class="flex size-12 items-center justify-center rounded-2xl bg-surface-muted">
        <IconSearchX v-if="keyword" class="size-5 text-muted" />
        <IconClipboard v-else class="size-5 text-muted" />
      </div>
      <p class="text-sm text-content-secondary">
        {{ keyword ? `没有与「${keyword}」匹配的记录` : "暂无剪贴板历史,复制任意文本后自动记录" }}
      </p>
    </div>
    <template v-else>
      <!-- 左栏:列表(单行预览),滚动到底分页加载 -->
      <div
        ref="listEl"
        class="flex w-2/5 shrink-0 flex-col overflow-y-auto border-r border-line p-2"
        @scroll="handleScroll"
      >
        <ClipboardListItem
          v-for="(item, index) in items"
          :key="item.id"
          :item="item"
          :selected="index === selectedIndex"
          @activate="paste(item)"
          @select="select(index)"
        />
      </div>
      <!-- 右栏:选中条目详情,头部放元信息与操作(键盘选中时也可达) -->
      <div v-if="selected" class="flex min-w-0 flex-1 flex-col">
        <header
          class="flex h-11 shrink-0 items-center justify-between gap-3 border-b border-line pr-2 pl-4"
        >
          <span class="truncate text-xs text-muted">{{ detailMeta }}</span>
          <span class="flex shrink-0 items-center gap-1">
            <button
              type="button"
              class="flex size-7 items-center justify-center rounded-md hover:bg-surface-muted"
              :title="copiedId === selected.id ? '已复制' : '仅复制,不粘贴'"
              @mousedown.prevent
              @click="copy(selected)"
            >
              <IconCheck v-if="copiedId === selected.id" class="size-4 text-content-secondary" />
              <IconCopy v-else class="size-4 text-muted" />
            </button>
            <button
              type="button"
              class="flex size-7 items-center justify-center rounded-md hover:bg-surface-muted"
              title="删除这条记录"
              @mousedown.prevent
              @click="remove(selected)"
            >
              <IconTrash2 class="size-4 text-muted" />
            </button>
          </span>
        </header>
        <div class="min-h-0 flex-1 overflow-y-auto p-4">
          <p class="cursor-text text-sm break-words whitespace-pre-wrap text-content select-text">
            {{ detailText }}
          </p>
          <p v-if="detailTruncated" class="mt-3 text-xs text-muted">
            内容过长,仅显示前 {{ DETAIL_MAX_CHARS }} 字符
          </p>
        </div>
      </div>
    </template>
  </div>
</template>
