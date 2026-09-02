<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import KeyboardKey from "@/launcher/components/KeyboardKey.vue";
import SearchInput from "@/launcher/components/SearchInput.vue";
import { useToolView } from "@/launcher/composables/useToolView";
import { iconOf } from "@/tools/icons";

const { activeModule, toolQuery, close } = useToolView();

/** 徽章上的工具图标,与磁贴共用登记表 */
const toolIcon = computed(() => (activeModule.value ? iconOf(activeModule.value.item.icon) : null));

const placeholder = computed(() => activeModule.value?.placeholder ?? "输入以过滤…");

const input = useTemplateRef<InstanceType<typeof SearchInput>>("input");

/** uTools 式肌肉记忆:输入框已空时再按退格,退出当前工具页 */
function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Backspace" && !event.isComposing && toolQuery.value === "") {
    event.preventDefault();
    close();
  }
}

defineExpose({ focus: () => input.value?.focus() });
</script>

<!-- 工具页搜索栏:工具徽章 + 页内过滤词,状态与退出动作都来自 useToolView。
     v-if 仅为收窄 activeModule 类型,实际只在工具页态被渲染 -->
<template>
  <div
    v-if="activeModule"
    data-tauri-drag-region
    class="flex h-16 shrink-0 items-center gap-3 px-5"
  >
    <!-- 工具徽章,点击退出(Esc / 空框退格同效) -->
    <button
      type="button"
      class="flex shrink-0 items-center gap-1.5 rounded-lg bg-muted px-2.5 py-1.5 text-sm text-foreground"
      title="退出(Esc)"
      @mousedown.prevent
      @click="close"
    >
      <component :is="toolIcon" class="size-4" />
      {{ activeModule.item.title }}
    </button>
    <SearchInput
      ref="input"
      v-model="toolQuery"
      :placeholder="placeholder"
      @keydown="handleKeydown"
    />
    <div class="flex shrink-0 items-center gap-1" title="全局唤起 / 收起快捷键">
      <KeyboardKey>Alt</KeyboardKey>
      <KeyboardKey>Enter</KeyboardKey>
    </div>
  </div>
</template>
