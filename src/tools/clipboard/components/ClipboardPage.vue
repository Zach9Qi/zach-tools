<script setup lang="ts">
import { computed, nextTick, toRef, useTemplateRef, watch } from "vue";
import IconClipboard from "~icons/lucide/clipboard";
import IconSearchX from "~icons/lucide/search-x";
import IconStar from "~icons/lucide/star";
import ClipboardDetailPane from "@/tools/clipboard/components/ClipboardDetailPane.vue";
import ClipboardFilterTabs from "@/tools/clipboard/components/ClipboardFilterTabs.vue";
import ClipboardListItem from "@/tools/clipboard/components/ClipboardListItem.vue";
import { useClipboardPage } from "@/tools/clipboard/composables/useClipboardPage";

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
  kindTabs,
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
} = useClipboardPage(toRef(props, "query"));

/** 过滤词(去空白),空态文案据此区分「无历史」与「无匹配」 */
const keyword = computed(() => props.query.trim());

/** 空态图标与文案:关键字无匹配 / 收藏空 / 无历史三种情形 */
const emptyIcon = computed(() =>
  keyword.value ? IconSearchX : favoriteOnly.value ? IconStar : IconClipboard,
);
const emptyHint = computed(() => {
  if (keyword.value) {
    return `没有与「${keyword.value}」匹配的记录`;
  }
  if (favoriteOnly.value) {
    return "还没有收藏,点亮条目的星标后常驻在这里";
  }
  return "暂无剪贴板历史,复制任意文本后自动记录";
});

// 键盘选中后让条目滚进可视区
watch(selectedIndex, async (index) => {
  await nextTick();
  listEl.value?.children[index]?.scrollIntoView({ block: "nearest" });
});

// 列表重置(输入过滤 / 切分类 / 藏窗口时漏了事件)后回到顶部;普通唤起不触发,滚动位置留着
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
  <div class="flex min-h-0 flex-1 flex-col">
    <ClipboardFilterTabs
      :tabs="kindTabs"
      :active-key="activeKind.key"
      :favorite-only="favoriteOnly"
      @select-kind="selectKind"
      @toggle-favorite-only="toggleFavoriteFilter"
    />
    <div class="flex min-h-0 flex-1 border-y border-border">
      <div
        v-if="items.length === 0 && !loading"
        class="flex flex-1 flex-col items-center justify-center gap-3"
      >
        <div class="flex size-12 items-center justify-center rounded-2xl bg-muted">
          <component :is="emptyIcon" class="size-5 text-muted-foreground" />
        </div>
        <p class="text-sm text-foreground">{{ emptyHint }}</p>
      </div>
      <template v-else>
        <!-- 左栏:列表(单行预览),滚动到底分页加载 -->
        <div
          ref="listEl"
          class="flex w-2/5 shrink-0 flex-col overflow-y-auto border-r border-border p-2"
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
        <!-- 右栏:选中条目详情 -->
        <ClipboardDetailPane
          v-if="selected"
          :item="selected"
          :copied="copiedId === selected.id"
          @copy="copy(selected)"
          @remove="remove(selected)"
          @toggle-favorite="toggleFavorite(selected)"
        />
      </template>
    </div>
  </div>
</template>
