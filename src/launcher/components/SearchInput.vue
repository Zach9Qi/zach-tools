<script setup lang="ts">
import { onMounted, ref } from "vue";

const query = defineModel<string>({ required: true });

const inputRef = ref<HTMLInputElement | null>(null);

/** 聚焦输入框。selectAll：窗口唤起时全选，输入即覆盖；切页带入过滤词时不全选，光标落到末尾接着改 */
function focusInput(selectAll: boolean) {
  const el = inputRef.value;
  if (!el) {
    return;
  }
  el.focus();
  if (selectAll) {
    el.select();
  } else {
    const end = el.value.length;
    el.setSelectionRange(end, end);
  }
}

/** 窗口唤起时由外壳调用：全选上次内容，直接输入即覆盖 */
function focus() {
  focusInput(true);
}

// 搜索栏随主页 ↔ 工具页切换而重建,挂载只聚焦不全选(匹配结果带入的过滤词要能接着改)
onMounted(() => focusInput(false));

defineExpose({ focus });
</script>

<!-- 纯输入框:只管样式与聚焦,placeholder / keydown 等由调用方经 attrs 透传 -->
<template>
  <input
    ref="inputRef"
    v-model="query"
    type="text"
    spellcheck="false"
    autocomplete="off"
    class="h-full min-w-0 flex-1 cursor-text bg-transparent text-lg text-content outline-hidden select-text placeholder:text-muted"
  />
</template>
