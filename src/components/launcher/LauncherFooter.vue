<script setup lang="ts">
import KeyboardKey from "@/components/launcher/KeyboardKey.vue";
</script>

<template>
  <footer class="flex h-10 shrink-0 items-center justify-end px-4">
    <div class="flex items-center gap-3 text-xs text-muted">
      <span class="flex items-center gap-1">
        <KeyboardKey>↑</KeyboardKey>
        <KeyboardKey>↓</KeyboardKey>
        选择
      </span>
      <span class="flex items-center gap-1">
        <KeyboardKey>↵</KeyboardKey>
        打开
      </span>
      <span class="flex items-center gap-1">
        <KeyboardKey>Esc</KeyboardKey>
        隐藏
      </span>
    </div>
  </footer>
</template>
