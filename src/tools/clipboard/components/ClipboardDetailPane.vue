<script setup lang="ts">
import { computed } from "vue";
import IconCheck from "~icons/lucide/check";
import IconCopy from "~icons/lucide/copy";
import IconStar from "~icons/lucide/star";
import IconTrash2 from "~icons/lucide/trash-2";
import { type ClipboardItem, toAssetUrl } from "@/tools/clipboard/lib/api";
import { formatRelativeTime } from "@/tools/clipboard/lib/time";

const props = defineProps<{
  /** 当前选中条目 */
  item: ClipboardItem;
  /** 「仅复制」刚成功,复制按钮短暂点亮反馈 */
  copied?: boolean;
}>();

const emit = defineEmits<{ copy: []; remove: []; toggleFavorite: [] }>();

/** 详情正文:后端已截断为最多 5000 字符的预览,原文不进前端 */
const detailText = computed(() => props.item.textPreview ?? "");
const truncated = computed(() => (props.item.textLength ?? 0) > detailText.value.length);

const isImage = computed(() => props.item.kind === "image");

/** 原图 URL;非 Tauri 运行时为空串,浏览器预览下仅不显示 */
const imageUrl = computed(() => (isImage.value ? toAssetUrl(props.item.imagePath) : ""));

/** 头部元信息:image 为像素尺寸,text 为原文字符数;均附相对时间 */
const meta = computed(() => {
  const time = formatRelativeTime(props.item.lastUsedAt);
  if (isImage.value) {
    return `图片 · ${props.item.imageWidth ?? 0}×${props.item.imageHeight ?? 0} · ${time}`;
  }
  const chars = props.item.textLength ?? detailText.value.length;
  return `文本 · ${chars} 字符 · ${time}`;
});
</script>

<!-- 选中条目详情:头部放元信息与操作(键盘选中时也可达),正文按类型渲染:image 居中等比预览原图,text 为文本预览 -->
<template>
  <div class="flex min-w-0 flex-1 flex-col">
    <header
      class="flex h-11 shrink-0 items-center justify-between gap-3 border-b border-border/80 pr-3 pl-4"
    >
      <span class="truncate text-xs font-medium text-muted-foreground/80">{{ meta }}</span>
      <span class="flex shrink-0 items-center gap-1">
        <button
          type="button"
          class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-all duration-150 hover:bg-accent hover:text-foreground active:scale-95"
          :title="item.isFavorite ? '取消收藏' : '收藏,常驻不被自动清理'"
          @mousedown.prevent
          @click="emit('toggleFavorite')"
        >
          <IconStar
            class="size-4 transition-colors"
            :class="item.isFavorite ? 'fill-warning text-warning' : ''"
          />
        </button>
        <button
          type="button"
          class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-all duration-150 hover:bg-accent hover:text-foreground active:scale-95"
          :title="copied ? '已复制' : '仅复制,不粘贴'"
          @mousedown.prevent
          @click="emit('copy')"
        >
          <IconCheck v-if="copied" class="size-4 text-foreground" />
          <IconCopy v-else class="size-4" />
        </button>
        <button
          type="button"
          class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-all duration-150 hover:bg-accent hover:text-destructive active:scale-95"
          title="删除这条记录"
          @mousedown.prevent
          @click="emit('remove')"
        >
          <IconTrash2 class="size-4" />
        </button>
      </span>
    </header>
    <!-- 图片容器不滚动:原图由 object-contain 等比缩进剩余高度,不溢出面板 -->
    <div
      v-if="isImage"
      class="flex min-h-0 flex-1 items-center justify-center overflow-hidden p-4.5"
    >
      <img :src="imageUrl" alt="" class="max-h-full max-w-full rounded-md object-contain" />
    </div>
    <div v-else class="min-h-0 flex-1 overflow-y-auto p-4.5">
      <p
        class="cursor-text text-sm/relaxed font-normal wrap-break-word whitespace-pre-wrap text-foreground select-text"
      >
        {{ detailText }}
      </p>
      <p v-if="truncated" class="mt-4 text-xs font-medium text-muted-foreground/70">
        内容过长,仅显示前 {{ detailText.length }} 字符
      </p>
    </div>
  </div>
</template>
