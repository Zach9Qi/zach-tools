<script setup lang="ts">
import { useTemplateRef } from "vue";
import LauncherFooter from "@/launcher/components/LauncherFooter.vue";
import ResultsPanel from "@/launcher/components/ResultsPanel.vue";
import SearchBar from "@/launcher/components/SearchBar.vue";
import { useLauncher } from "@/launcher/composables/useLauncher";

const searchBar = useTemplateRef<InstanceType<typeof SearchBar>>("searchBar");

const { query, hide } = useLauncher({
  onOpen: () => searchBar.value?.focus(),
});
</script>

<template>
  <!-- 外层透明边缘：给面板阴影留渲染空间，点击该区域收起启动器 -->
  <div class="flex h-full w-full p-5" @mousedown.self="hide">
    <!-- scheme-light-dark 只放在面板根：让 light-dark() token 与表单控件跟随系统主题，
         同时不影响 html 根，保证窗口透明边缘不被画上底色 -->
    <section
      class="flex min-w-0 flex-1 cursor-default flex-col overflow-hidden rounded-2xl bg-surface antialiased scheme-light-dark shadow-panel ring-1 ring-edge select-none"
    >
      <SearchBar ref="searchBar" v-model="query" />
      <ResultsPanel :query="query" />
      <LauncherFooter />
    </section>
  </div>
</template>
