import { ref } from "vue";
import type { ViewToolModule } from "@/tools/types";

/** 当前打开的工具页注册单元;null = 主页。模块级状态,所有组件共享同一份 */
const activeModule = ref<ViewToolModule | null>(null);

/** 工具页内过滤词;随 open 初始化、close 清空,与主页搜索词是两份状态,互不串扰 */
const toolQuery = ref("");

/**
 * 工具页视图状态:主页 ↔ 工具页的同窗口切换,以及工具页自己的过滤词。
 * 打开/关闭只改这一份状态,渲染(LauncherPanel)、工具页搜索栏(ToolSearchBar)、
 * 页脚 Esc 文案(LauncherFooter)各自读取,无需层层传 prop。
 * 窗口隐藏不重置(保留现场),退出只由 Esc / 徽章点击 / 空框退格触发。
 */
export function useToolView() {
  /** 进入工具页;initialQuery 仅在当前输入就是工具入参时传入(匹配结果进入),作页内过滤词 */
  function open(module: ViewToolModule, initialQuery = "") {
    toolQuery.value = initialQuery;
    activeModule.value = module;
  }

  /** 返回主页,页内过滤词一并放弃;主页搜索词是另一份状态,退出后主页保持原样 */
  function close() {
    activeModule.value = null;
    toolQuery.value = "";
  }

  return { activeModule, toolQuery, open, close };
}
