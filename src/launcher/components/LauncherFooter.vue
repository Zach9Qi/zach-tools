<script setup lang="ts">
import KeyboardKey from "@/launcher/components/KeyboardKey.vue";
import { useToolView } from "@/launcher/composables/useToolView";

const { activeModule } = useToolView();
</script>

<template>
  <footer class="flex h-10 shrink-0 items-center justify-end px-4">
    <div class="flex items-center gap-3 text-xs text-muted">
      <span class="flex items-center gap-1">
        <KeyboardKey>↑</KeyboardKey>
        <KeyboardKey>↓</KeyboardKey>
        <template v-if="!activeModule">
          <KeyboardKey>←</KeyboardKey>
          <KeyboardKey>→</KeyboardKey>
        </template>
        选择
      </span>
      <span class="flex items-center gap-1">
        <KeyboardKey>↵</KeyboardKey>
        打开
      </span>
      <!-- Esc 分层:工具页内是返回主页,主页才是隐藏窗口 -->
      <span class="flex items-center gap-1">
        <KeyboardKey>Esc</KeyboardKey>
        {{ activeModule ? "返回" : "隐藏" }}
      </span>
    </div>
  </footer>
</template>
