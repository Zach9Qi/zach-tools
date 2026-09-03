<script setup lang="ts">
import { computed } from "vue";
import { iconOf } from "@/tools/icons";
import type { ToolItem } from "@/tools/types";

const props = defineProps<{ item: ToolItem; selected?: boolean }>();

const emit = defineEmits<{ activate: []; select: [] }>();

const icon = computed(() => iconOf(props.item.icon));
</script>

<template>
  <!-- mousedown.prevent:点击磁贴不把焦点从搜索框抢走 -->
  <button
    type="button"
    class="group flex flex-col items-center gap-1.5 rounded-xl p-2 transition-all duration-150 ease-out active:scale-96"
    :class="selected ? 'bg-accent shadow-2xs' : 'hover:bg-accent/60'"
    @mousedown.prevent
    @mouseenter="emit('select')"
    @click="emit('activate')"
  >
    <span
      class="flex size-11 shrink-0 items-center justify-center rounded-xl border border-border/50 bg-muted shadow-2xs transition-all duration-150 group-hover:border-border group-hover:shadow-xs"
      :class="selected ? 'border-border' : ''"
    >
      <component
        :is="icon"
        class="size-5.5 text-foreground transition-transform duration-150 group-hover:scale-105"
      />
    </span>
    <!-- min-h 固定两行高度:名称一行或两行时磁贴等高;超两行截断 -->
    <span
      class="line-clamp-2 min-h-8 w-full text-center text-xs/4 font-medium text-foreground transition-colors"
      :class="selected ? 'text-accent-foreground' : ''"
    >
      {{ item.title }}
    </span>
  </button>
</template>
