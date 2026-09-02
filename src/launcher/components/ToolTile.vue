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
    class="flex flex-col items-center gap-1.5 rounded-xl p-2"
    :class="selected ? 'bg-accent' : ''"
    @mousedown.prevent
    @mouseenter="emit('select')"
    @click="emit('activate')"
  >
    <span class="flex size-11 shrink-0 items-center justify-center rounded-lg bg-muted">
      <component :is="icon" class="size-5 text-foreground" />
    </span>
    <!-- min-h 固定两行高度:名称一行或两行时磁贴等高;超两行截断 -->
    <span class="line-clamp-2 min-h-8 w-full text-center text-xs/4 text-foreground">
      {{ item.title }}
    </span>
  </button>
</template>
