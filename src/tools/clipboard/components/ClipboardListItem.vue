<script setup lang="ts">
import { computed, type Component } from "vue";
import IconFiles from "~icons/lucide/files";
import IconImage from "~icons/lucide/image";
import IconStar from "~icons/lucide/star";
import IconType from "~icons/lucide/type";
import { type ClipboardItem, type ClipboardKind } from "@/tools/clipboard/lib/api";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const props = defineProps<{ item: ClipboardItem; selected?: boolean }>();

const emit = defineEmits<{ activate: []; select: [] }>();

/** 类型徽标图标;图片条目改为渲染真缩略图是二期(图片入库)工作 */
const KIND_ICONS: Record<ClipboardKind, Component> = {
  text: IconType,
  image: IconImage,
  files: IconFiles,
};

const kindIcon = computed(() => KIND_ICONS[props.item.kind]);

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
    class="flex h-11 shrink-0 cursor-default items-center gap-2.5 rounded-lg px-2.5 transition-colors duration-100 ease-out"
    :class="selected ? 'bg-accent font-medium shadow-2xs' : 'hover:bg-accent/50'"
    @mousedown.prevent
    @mouseenter="emit('select')"
    @click="emit('activate')"
  >
    <span
      class="flex size-7 shrink-0 items-center justify-center rounded-md border border-border/40 bg-muted/80 shadow-2xs"
    >
      <component :is="kindIcon" class="size-3.5 text-muted-foreground" />
    </span>
    <span
      class="min-w-0 flex-1 truncate text-sm transition-colors"
      :class="selected ? 'text-accent-foreground' : 'text-foreground'"
    >
      {{ preview }}
    </span>
    <IconStar v-if="item.isFavorite" class="size-3.5 shrink-0 fill-warning text-warning" />
    <span class="shrink-0 text-xs text-muted-foreground/80">{{ time }}</span>
  </div>
</template>
