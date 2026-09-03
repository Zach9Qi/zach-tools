<script setup lang="ts">
import { computed, type Component } from "vue";
import IconFiles from "~icons/lucide/files";
import IconImage from "~icons/lucide/image";
import IconStar from "~icons/lucide/star";
import IconType from "~icons/lucide/type";
import { type ClipboardItem, type ClipboardKind, toAssetUrl } from "@/tools/clipboard/lib/api";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const props = defineProps<{ item: ClipboardItem; selected?: boolean }>();

const emit = defineEmits<{ activate: []; select: [] }>();

/** 类型徽标图标;image 条目在同一格内渲染缩略图,图标仅作缩略图路径缺失时的兜底 */
const KIND_ICONS: Record<ClipboardKind, Component> = {
  text: IconType,
  image: IconImage,
  files: IconFiles,
};

const kindIcon = computed(() => KIND_ICONS[props.item.kind]);

/** 缩略图 URL;非 image 条目或非 Tauri 运行时为空,模板据此回退到图标 */
const thumbnailUrl = computed(() =>
  props.item.kind === "image" ? toAssetUrl(props.item.thumbnailPath) : "",
);

/** 单行预览截取长度:够填满一行即可,不把整段预览(最多 5000 字符)塞进 DOM */
const PREVIEW_MAX_CHARS = 200;

/** 单行预览:image 显示像素尺寸;text 先截断再把换行等空白折叠成空格 */
const preview = computed(() => {
  if (props.item.kind === "image") {
    return `图片 · ${props.item.imageWidth ?? 0}×${props.item.imageHeight ?? 0}`;
  }
  return (props.item.textPreview ?? "").slice(0, PREVIEW_MAX_CHARS).replace(/\s+/g, " ").trim();
});

const time = computed(() => formatRelativeTime(props.item.lastUsedAt));
</script>

<template>
  <!-- mousedown.prevent:点击条目不把焦点从搜索框抢走;整行点击即粘贴,操作按钮在右栏详情头部 -->
  <div
    class="flex h-11 shrink-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 transition-colors duration-100 ease-out"
    :class="selected ? 'bg-accent font-medium shadow-2xs' : 'hover:bg-accent/50'"
    @mousedown.prevent
    @mouseenter="emit('select')"
    @click="emit('activate')"
  >
    <!-- 图标格固定 28×28:image 条目用缩略图填满(object-cover 裁切),行高保持 44px 不变 -->
    <span
      class="flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border/40 bg-muted/80 shadow-2xs"
    >
      <img v-if="thumbnailUrl" :src="thumbnailUrl" alt="" class="size-7 object-cover" />
      <component v-else :is="kindIcon" class="size-3.5 text-muted-foreground" />
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
