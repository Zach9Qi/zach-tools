<script setup lang="ts">
import { computed } from "vue";
import IconCheck from "~icons/lucide/check";
import IconCopy from "~icons/lucide/copy";
import IconStar from "~icons/lucide/star";
import IconTrash2 from "~icons/lucide/trash-2";
import { type ClipboardItem } from "@/tools/clipboard/lib/api";
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

/** 头部元信息:原文字符数 + 相对时间 */
const meta = computed(() => {
  const chars = props.item.textLength ?? detailText.value.length;
  return `文本 · ${chars} 字符 · ${formatRelativeTime(props.item.lastUsedAt)}`;
});
</script>

<!-- 选中条目详情:头部放元信息与操作(键盘选中时也可达),正文为文本预览 -->
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
    <div class="min-h-0 flex-1 overflow-y-auto p-4.5">
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
