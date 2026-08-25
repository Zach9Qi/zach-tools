<script setup lang="ts">
import { useTemplateRef } from "vue";
import HomeSearchBar from "@/launcher/components/HomeSearchBar.vue";
import LauncherFooter from "@/launcher/components/LauncherFooter.vue";
import ResultsPanel from "@/launcher/components/ResultsPanel.vue";
import ToolSearchBar from "@/launcher/components/ToolSearchBar.vue";
import { useAutoHeight } from "@/launcher/composables/useAutoHeight";
import { useLauncher } from "@/launcher/composables/useLauncher";
import type { SectionKey } from "@/launcher/composables/useResults";
import { useToolView } from "@/launcher/composables/useToolView";
import { moduleOf } from "@/tools/registry";
import { isViewModule, type ToolItem } from "@/tools/types";

/** 当前渲染的搜索栏(主页 / 工具页二选一,共用 ref 名),窗口唤起时聚焦用 */
const searchBar = useTemplateRef<{ focus: () => void }>("searchBar");
const root = useTemplateRef<HTMLElement>("root");

const { homeQuery, hide } = useLauncher({
  onOpen: () => searchBar.value?.focus(),
});

const { activeModule, toolQuery, open } = useToolView();

// 窗口高度跟随根元素(面板 + 阴影边距),上限由面板的 CSS max-h 封顶
useAutoHeight(root);

/**
 * 打开工具:按 id 查注册单元,view 型切入工具页、launch 型执行动作。
 * view 型进入时,仅内容匹配(matches)的搜索词作为过滤词带入工具页,
 * 名称命中 / 主页点入则从空开始;主页搜索词不动,退出工具页后主页保持原样。
 */
async function activate(tool: ToolItem, source: SectionKey) {
  const module = moduleOf(tool);
  if (isViewModule(module)) {
    open(module, source === "matches" ? homeQuery.value : "");
    return;
  }
  await module.run({ query: homeQuery.value });
}
</script>

<template>
  <!-- 外层透明边缘：给面板阴影留渲染空间，点击该区域收起启动器。
       不设 h-full：高度由内容决定，窗口高度直接贴这一层（p-5 边距已含在内） -->
  <div ref="root" class="flex w-full p-5" @mousedown.self="hide">
    <!-- scheme-light-dark 只放在面板根：让 light-dark() token 与表单控件跟随系统主题，
         同时不影响 html 根，保证窗口透明边缘不被画上底色 -->
    <!-- max-h 是窗口高度上限的唯一定义处：封顶后内部区域滚动，窗口不再变高。
         工具页态强制撑满到上限：列表在固定视口内滚动，打字过滤时窗口高度不抖动 -->
    <section
      class="flex max-h-128 min-w-0 flex-1 cursor-default flex-col overflow-hidden rounded-2xl bg-surface antialiased scheme-light-dark shadow-panel ring-1 ring-edge select-none"
      :class="activeModule ? 'h-128' : ''"
    >
      <!-- 主页与工具页各自成组:搜索栏随态切换重建,挂载时自动聚焦 -->
      <template v-if="activeModule">
        <ToolSearchBar ref="searchBar" />
        <component :is="activeModule.page" :query="toolQuery" />
      </template>
      <template v-else>
        <HomeSearchBar ref="searchBar" v-model="homeQuery" />
        <ResultsPanel :query="homeQuery" @activate="activate" />
      </template>
      <LauncherFooter />
    </section>
  </div>
</template>
