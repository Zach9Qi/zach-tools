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

const { homeQuery } = useLauncher({
  onOpen: () => searchBar.value?.focus(),
});

const { activeModule, toolQuery, open } = useToolView();

// 窗口高度跟随面板根元素,上限由面板的 CSS max-h 封顶
useAutoHeight(root);

/**
 * 打开工具:按 id 查注册单元,view 型切入工具页、launch 型执行动作。
 * 内容匹配进入:主页搜索词先带入工具搜索栏作过滤词;名称命中 / 主页点入则从空开始。
 * view 型一旦打开就把主页搜索词清空,退出不写回,回到主页是空白搜索。
 */
async function activate(tool: ToolItem, source: SectionKey) {
  const module = moduleOf(tool);
  if (isViewModule(module)) {
    open(module, source === "matches" ? homeQuery.value : "");
    homeQuery.value = "";
    return;
  }
  await module.run({ query: homeQuery.value });
}
</script>

<template>
  <!-- 面板就是页面根：无阴影、无外边距，窗口尺寸直接贴这一层。
       不设 h-full：高度由内容决定；max-h 是窗口高度上限的唯一定义处，封顶后内部区域滚动。
       工具页态强制撑满到上限：列表在固定视口内滚动，打字过滤时窗口高度不抖动。
       描边用 border 而非 ring：ring 画在元素外侧，面板贴窗口边会被裁掉。
       scheme-light-dark 只放在面板根：让 light-dark() 原始变量与表单控件跟随系统主题，
       同时不影响 html 根，圆角外的透明角不被画上底色；body 不设底色。 -->
  <section
    ref="root"
    class="flex max-h-150 w-full flex-col overflow-hidden rounded-2xl border border-border bg-background text-foreground scheme-light-dark"
    :class="activeModule ? 'h-150' : ''"
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
</template>
