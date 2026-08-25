<script setup lang="ts">
import { computed, ref } from "vue";
import IconSearch from "~icons/lucide/search";
import KeyboardKey from "@/launcher/components/KeyboardKey.vue";
import { useToolView } from "@/launcher/composables/useToolView";
import { iconOf } from "@/tools/icons";

const query = defineModel<string>({ required: true });

const { activeModule, close } = useToolView();

/** 工具页态徽章上的图标,与磁贴共用登记表 */
const toolIcon = computed(() => (activeModule.value ? iconOf(activeModule.value.item.icon) : null));

const placeholder = computed(() => activeModule.value?.placeholder ?? "搜索应用、文件和插件功能…");

const inputRef = ref<HTMLInputElement | null>(null);

/** 聚焦搜索框并全选上次搜索词：窗口唤起后直接输入即覆盖，无需手动清空 */
function focus() {
  inputRef.value?.focus();
  inputRef.value?.select();
}

/** 退出工具页并放弃页内过滤词(徽章点击 / 空框退格) */
function exitTool() {
  close();
  query.value = "";
}

/** uTools 式肌肉记忆:输入框已空时再按退格,退出当前工具页 */
function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Backspace" && !event.isComposing && query.value === "" && activeModule.value) {
    event.preventDefault();
    exitTool();
  }
}

defineExpose({ focus });
</script>

<template>
  <!-- data-tauri-drag-region：按住搜索栏空白处可拖动窗口 -->
  <div data-tauri-drag-region class="flex h-16 shrink-0 items-center gap-3 px-5">
    <!-- 主页态是搜索图标;工具页态换成工具徽章,点击退出(Esc / 空框退格同效) -->
    <IconSearch v-if="!activeModule" class="size-5 shrink-0 text-muted" />
    <button
      v-else
      type="button"
      class="flex shrink-0 items-center gap-1.5 rounded-lg bg-surface-muted px-2.5 py-1.5 text-sm text-content-secondary"
      title="退出(Esc)"
      @mousedown.prevent
      @click="exitTool"
    >
      <component :is="toolIcon" class="size-4" />
      {{ activeModule.item.title }}
    </button>
    <input
      ref="inputRef"
      v-model="query"
      type="text"
      autofocus
      spellcheck="false"
      autocomplete="off"
      :placeholder="placeholder"
      class="h-full min-w-0 flex-1 cursor-text bg-transparent text-lg text-content outline-hidden select-text placeholder:text-muted"
      @keydown="handleKeydown"
    />
    <div class="flex shrink-0 items-center gap-1" title="全局唤起 / 收起快捷键">
      <KeyboardKey>Alt</KeyboardKey>
      <KeyboardKey>Enter</KeyboardKey>
    </div>
  </div>
</template>
