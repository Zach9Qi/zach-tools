<script setup lang="ts">
import { useKeymap } from "@/composables/useKeymap";
import KeyboardKey from "@/launcher/components/KeyboardKey.vue";
import { useToolView } from "@/launcher/composables/useToolView";

const { activeModule } = useToolView();

// 提示由当前页面登记的快捷键表派生:主页是结果区的行导航,工具页是各自的登记,页脚不维护副本
const { hints } = useKeymap();
</script>

<template>
  <footer
    class="flex h-10 shrink-0 items-center justify-end border-t border-border/40 bg-muted/20 px-4"
  >
    <div class="flex items-center gap-3.5 text-xs text-muted-foreground/90">
      <span v-for="hint in hints" :key="hint.label" class="flex items-center gap-1.5">
        <KeyboardKey v-for="key in hint.keys" :key="key">{{ key }}</KeyboardKey>
        <span class="text-muted-foreground">{{ hint.label }}</span>
      </span>
      <!-- Esc 归外壳,不进键表:工具页内是返回主页,主页才是隐藏窗口 -->
      <span class="flex items-center gap-1.5">
        <KeyboardKey>Esc</KeyboardKey>
        <span class="text-muted-foreground">{{ activeModule ? "返回" : "隐藏" }}</span>
      </span>
    </div>
  </footer>
</template>
