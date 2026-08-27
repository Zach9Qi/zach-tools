<script setup lang="ts">
import { computed } from "vue";
import { type ClipboardItem } from "@/tools/clipboard/lib/api";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const props = defineProps<{ item: ClipboardItem; selected?: boolean }>();

const emit = defineEmits<{ activate: []; select: [] }>();

/** 单行预览截取长度:够填满一行即可,不把整段预览(最多 5000 字符)塞进 DOM */
const PREVIEW_MAX_CHARS = 200;

/** 单行预览:先截断再把换行等空白折叠成空格 */
const preview = computed(() =>
  (props.item.textPreview ?? "").slice(0, PREVIEW_MAX_CHARS).replace(/\s+/g, " ").trim(),
);

const time = computed(() => formatRelativeTime(props.item.lastUsedAt));
</script>

<template>
  <!-- mousedown.prevent:点击条目不把焦点从搜索框抢走;整行点击即粘贴,操作按钮在右栏详情头部 -->
  <div
    class="flex h-11 shrink-0 cursor-default items-center gap-3 rounded-lg px-3"
    :class="selected ? 'bg-surface-muted' : ''"
    @mousedown.prevent
    @mouseenter="emit('select')"
    @click="emit('activate')"
  >
    <span class="min-w-0 flex-1 truncate text-sm text-content">{{ preview }}</span>
    <span class="shrink-0 text-xs text-muted">{{ time }}</span>
  </div>
</template>
