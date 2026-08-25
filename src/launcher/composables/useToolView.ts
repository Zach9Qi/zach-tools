import { ref } from "vue";
import type { ViewToolModule } from "@/tools/types";

/** 当前打开的工具页注册单元;null = 主页。模块级状态,所有组件共享同一份 */
const activeModule = ref<ViewToolModule | null>(null);

/**
 * 工具页视图状态:主页 ↔ 工具页的同窗口切换。
 * 打开/关闭只改这一份状态,渲染(LauncherPanel)、搜索框徽章与 placeholder(SearchBar)、
 * 页脚 Esc 文案(LauncherFooter)各自读取,无需层层传 prop。
 * 窗口隐藏不重置(保留现场),退出只由 Esc / 徽章点击 / 空框退格触发。
 */
export function useToolView() {
  /** 进入工具页 */
  function open(module: ViewToolModule) {
    activeModule.value = module;
  }

  /** 返回主页 */
  function close() {
    activeModule.value = null;
  }

  return { activeModule, open, close };
}
