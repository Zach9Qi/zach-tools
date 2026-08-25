import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { useToolView } from "@/launcher/composables/useToolView";
import { hideLauncher, onLauncherOpen } from "@/launcher/lib/window";

/** useLauncher 可选配置 */
export interface UseLauncherOptions {
  /** 窗口每次唤起后调用（用于聚焦搜索框等） */
  onOpen?: () => void;
}

/**
 * 启动器窗口级状态编排：
 * 维护主页搜索词、响应后端 launcher-open 事件、处理 Esc 分层收起。
 * 收起只是隐藏窗口，搜索词与所在页面等状态保留，下次唤起接着上次的样子。
 */
export function useLauncher(options: UseLauncherOptions = {}) {
  /** 主页全局搜索词;工具页内过滤词是另一份状态(useToolView 的 toolQuery),互不串扰 */
  const homeQuery = ref("");

  const toolView = useToolView();

  /** 收起启动器（调用后端命令隐藏窗口） */
  function hide() {
    void hideLauncher();
  }

  function handleKeydown(event: KeyboardEvent) {
    // isComposing：中文输入法组词时按 Esc 只取消组词，不收起窗口
    if (event.key !== "Escape" || event.isComposing) {
      return;
    }
    // Esc 分层：工具页内先返回主页（close 内部放弃过滤词，主页现场保持原样），主页才隐藏窗口
    if (toolView.activeModule.value) {
      toolView.close();
      return;
    }
    hide();
  }

  let unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    window.addEventListener("keydown", handleKeydown);
    unlisteners = await Promise.all([
      // 唤起时不清搜索词：保留上次现场，搜索框聚焦时全选，输入即覆盖
      onLauncherOpen(() => {
        options.onOpen?.();
      }),
    ]);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });

  return { homeQuery, hide };
}
