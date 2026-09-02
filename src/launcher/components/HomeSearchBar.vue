<script setup lang="ts">
import { useTemplateRef } from "vue";
import IconSearch from "~icons/lucide/search";
import KeyboardKey from "@/launcher/components/KeyboardKey.vue";
import SearchInput from "@/launcher/components/SearchInput.vue";

/** 主页全局搜索词;主页态下唯一的写入方就是这个输入框 */
const query = defineModel<string>({ required: true });

const input = useTemplateRef<InstanceType<typeof SearchInput>>("input");

defineExpose({ focus: () => input.value?.focus() });
</script>

<!-- 主页搜索栏:搜索图标 + 全局搜索词;data-tauri-drag-region 按住空白处可拖动窗口 -->
<template>
  <div data-tauri-drag-region class="flex h-16 shrink-0 items-center gap-3 px-5">
    <IconSearch class="size-5 shrink-0 text-muted-foreground" />
    <SearchInput ref="input" v-model="query" placeholder="搜索应用、文件和插件功能…" />
    <div class="flex shrink-0 items-center gap-1" title="全局唤起 / 收起快捷键">
      <KeyboardKey>Alt</KeyboardKey>
      <KeyboardKey>Enter</KeyboardKey>
    </div>
  </div>
</template>
