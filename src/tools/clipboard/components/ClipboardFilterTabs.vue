<script setup lang="ts">
import IconStar from "~icons/lucide/star";
import type { ClipboardKindTab } from "@/tools/clipboard/lib/tabs";

defineProps<{
  /** 类型 tab(注册表,单选) */
  tabs: ClipboardKindTab[];
  /** 当前选中类型 tab 的 key */
  activeKey: string;
  /** 「只看收藏」开关状态,与类型 tab 正交叠加 */
  favoriteOnly: boolean;
}>();

const emit = defineEmits<{ selectKind: [key: string]; toggleFavoriteOnly: [] }>();
</script>

<!-- 过滤行:左侧类型 tab 单选,分隔线右侧是正交的「收藏」开关;
     mousedown.prevent 保持焦点在搜索框 -->
<template>
  <div class="flex h-10 shrink-0 items-center gap-1 border-t border-line px-3">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      type="button"
      class="rounded-lg px-2.5 py-1.5 text-xs"
      :class="
        tab.key === activeKey
          ? 'bg-surface-muted text-content'
          : 'text-muted hover:text-content-secondary'
      "
      @mousedown.prevent
      @click="emit('selectKind', tab.key)"
    >
      {{ tab.label }}
    </button>
    <span class="mx-1 h-4 w-px shrink-0 bg-line" />
    <button
      type="button"
      class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs"
      :class="
        favoriteOnly ? 'bg-surface-muted text-content' : 'text-muted hover:text-content-secondary'
      "
      :title="favoriteOnly ? '显示全部条目' : '只看收藏(Ctrl+F)'"
      @mousedown.prevent
      @click="emit('toggleFavoriteOnly')"
    >
      <IconStar class="size-3" :class="favoriteOnly ? 'fill-amber-400 text-amber-400' : ''" />
      收藏
    </button>
  </div>
</template>
