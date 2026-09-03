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
  <div class="flex h-11 shrink-0 items-center justify-between border-t border-border/80 px-4">
    <!-- macOS 风格分段选择器 (Segmented Control) 质感 -->
    <div
      class="flex items-center gap-0.5 rounded-lg border border-border/40 bg-muted/50 p-0.5 shadow-2xs"
    >
      <button
        v-for="tab in tabs"
        :key="tab.key"
        type="button"
        class="rounded-md px-3 py-1 text-xs font-medium transition-all duration-150"
        :class="
          tab.key === activeKey
            ? 'bg-background text-foreground shadow-xs'
            : 'text-muted-foreground hover:text-foreground'
        "
        @mousedown.prevent
        @click="emit('selectKind', tab.key)"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- 收藏过滤开关 -->
    <button
      type="button"
      class="flex items-center gap-1.5 rounded-lg border px-2.5 py-1 text-xs font-medium transition-all duration-150"
      :class="
        favoriteOnly
          ? 'border-border/60 bg-accent text-accent-foreground shadow-2xs'
          : 'border-transparent text-muted-foreground hover:bg-muted/60 hover:text-foreground'
      "
      :title="favoriteOnly ? '显示全部条目' : '只看收藏(Ctrl+F)'"
      @mousedown.prevent
      @click="emit('toggleFavoriteOnly')"
    >
      <IconStar
        class="size-3.5 transition-colors"
        :class="favoriteOnly ? 'fill-warning text-warning' : 'text-muted-foreground'"
      />
      收藏
    </button>
  </div>
</template>
