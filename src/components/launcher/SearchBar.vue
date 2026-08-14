<script setup lang="ts">
import { ref } from "vue";
import IconSearch from "~icons/lucide/search";
import KeyboardKey from "@/components/launcher/KeyboardKey.vue";

const query = defineModel<string>({ required: true });

const inputRef = ref<HTMLInputElement | null>(null);

/** 聚焦搜索框，供父组件在窗口唤起后调用 */
function focus() {
  inputRef.value?.focus();
}

defineExpose({ focus });
</script>

<template>
  <!-- data-tauri-drag-region：按住搜索栏空白处可拖动窗口 -->
  <div data-tauri-drag-region class="flex h-16 shrink-0 items-center gap-3 px-5">
    <IconSearch class="size-5 shrink-0 text-muted" />
    <input
      ref="inputRef"
      v-model="query"
      type="text"
      autofocus
      spellcheck="false"
      autocomplete="off"
      placeholder="搜索应用、文件和插件功能…"
      class="h-full min-w-0 flex-1 cursor-text bg-transparent text-lg text-content outline-hidden select-text placeholder:text-muted"
    />
    <div class="flex shrink-0 items-center gap-1" title="全局唤起 / 收起快捷键">
      <KeyboardKey>Alt</KeyboardKey>
      <KeyboardKey>Enter</KeyboardKey>
    </div>
  </div>
</template>
