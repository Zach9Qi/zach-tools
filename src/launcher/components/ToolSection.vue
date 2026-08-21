<script setup lang="ts">
import ToolTile from "@/launcher/components/ToolTile.vue";
import type { SectionEntry } from "@/launcher/composables/useResults";
import type { ToolItem } from "@/tools/types";

defineProps<{
  /** 分区标题(最近使用 / 已固定 / 匹配结果 / 搜索结果) */
  title: string;
  /** 本区条目,全局下标已由数据层盖章;每行 8 个、超出由网格换行;调用方负责裁切,这里不滚动 */
  entries: SectionEntry[];
  /** 全局选中下标;与条目的 index 相等则高亮,不在本区时本区无高亮 */
  selectedIndex?: number;
}>();

const emit = defineEmits<{ activate: [tool: ToolItem]; select: [index: number] }>();
</script>

<template>
  <section class="flex flex-col gap-1">
    <header class="flex h-5 items-center justify-between px-2">
      <h2 class="text-xs font-medium text-muted">{{ title }}</h2>
      <!-- 右侧操作槽(如「全部 >」),不传则无 -->
      <slot name="action" />
    </header>
    <div class="grid grid-cols-8 gap-1">
      <ToolTile
        v-for="entry in entries"
        :key="entry.tool.id"
        :item="entry.tool"
        :selected="entry.index === selectedIndex"
        @activate="emit('activate', entry.tool)"
        @select="emit('select', entry.index)"
      />
    </div>
  </section>
</template>
