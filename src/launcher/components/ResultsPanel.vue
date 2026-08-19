<script setup lang="ts">
import IconSearchX from "~icons/lucide/search-x";
import IconSparkles from "~icons/lucide/sparkles";

defineProps<{ query: string }>();
</script>

<template>
  <!-- 结果区空壳：后续搜索结果、插件列表接入后替换这里的空态 -->
  <div
    class="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 border-y border-line px-8"
  >
    <div class="flex size-14 items-center justify-center rounded-2xl bg-surface-muted">
      <IconSearchX v-if="query" class="size-6 text-muted" />
      <IconSparkles v-else class="size-6 text-muted" />
    </div>
    <div class="flex flex-col items-center gap-1 text-center">
      <p class="text-sm font-medium text-content-secondary">
        {{ query ? `没有与「${query}」匹配的内容` : "输入关键词开始搜索" }}
      </p>
      <p class="text-xs text-muted">
        {{
          query ? "搜索能力接入后，匹配结果将展示在这里" : "应用、文件和插件功能接入后将展示在这里"
        }}
      </p>
    </div>
  </div>
</template>
