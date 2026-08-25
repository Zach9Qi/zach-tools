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
 * 维护搜索词、响应后端 launcher-open 事件、处理 Esc 分层收起。
 * 收起只是隐藏窗口，搜索词与所在页面等状态保留，下次唤起接着上次的样子。
 */
export function useLauncher(options: UseLauncherOptions = {}) {
  /** 当前搜索词;主页是全局搜索词,工具页内是页内过滤词 */
  const query = ref("");

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
    // Esc 分层：工具页内先返回主页（过滤词一并放弃），主页才隐藏窗口
    if (toolView.activeModule.value) {
      toolView.close();
      query.value = "";
      return;
    }
    hide();
  }

  let unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    window.addEventListener("keydown", handleKeydown);
    unlisteners = await Promise.all([
      // 唤起时不清 query：保留上次搜索词，SearchBar 聚焦时全选，输入即覆盖
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

  return { query, hide };
}
