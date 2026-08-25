<script setup lang="ts">
import { toRef } from "vue";
import IconChevronRight from "~icons/lucide/chevron-right";
import IconSearchX from "~icons/lucide/search-x";
import ToolSection from "@/launcher/components/ToolSection.vue";
import { useResults, type SectionKey } from "@/launcher/composables/useResults";
import type { ToolItem } from "@/tools/types";

const props = defineProps<{ query: string }>();

const emit = defineEmits<{ activate: [tool: ToolItem, source: SectionKey] }>();

const { isSearch, sections, selectedIndex, select } = useResults({
  query: toRef(props, "query"),
  onActivate: (tool, source) => emit("activate", tool, source),
});
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col border-y border-line p-4">
    <!-- 搜索态全部分区为空才提示无结果;剪贴板列表接入前只看磁贴分区 -->
    <div v-if="isSearch && sections.length === 0" class="flex flex-col items-center gap-3 py-10">
      <div class="flex size-12 items-center justify-center rounded-2xl bg-surface-muted">
        <IconSearchX class="size-5 text-muted" />
      </div>
      <p class="text-sm text-content-secondary">没有与「{{ query }}」匹配的内容</p>
    </div>

    <!-- 分区由 useResults 的 sections 驱动:渲染顺序、offset 与导航展平同源,模板不再手工对齐。
         磁贴区 shrink-0:剪贴板列表接入后排在上方,面板到高度上限时列表内部滚动,磁贴不滚走 -->
    <div v-else class="flex shrink-0 flex-col gap-4">
      <ToolSection
        v-for="section in sections"
        :key="section.key"
        :title="section.title"
        :entries="section.entries"
        :selected-index="selectedIndex"
        @activate="emit('activate', $event.tool, $event.section)"
        @select="select"
      >
        <!-- 已固定分区特有的「全部 >」;第一版只占位,不跳转 -->
        <template v-if="section.key === 'pinned'" #action>
          <span class="flex items-center gap-0.5 text-xs text-muted">
            全部
            <IconChevronRight class="size-3" />
          </span>
        </template>
      </ToolSection>
    </div>
  </div>
</template>
