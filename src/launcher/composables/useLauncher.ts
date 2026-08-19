import { onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { hideLauncher, onLauncherClose, onLauncherOpen } from "@/launcher/lib/window";

/** useLauncher 可选配置 */
export interface UseLauncherOptions {
  /** 窗口每次唤起后调用（用于聚焦搜索框等） */
  onOpen?: () => void;
}

/**
 * 启动器窗口级状态编排：
 * 维护搜索词、响应后端 launcher-open / launcher-close 事件、处理 Esc 收起。
 */
export function useLauncher(options: UseLauncherOptions = {}) {
  /** 当前搜索词 */
  const query = ref("");

  /** 收起启动器（调用后端命令隐藏窗口） */
  function hide() {
    void hideLauncher();
  }

  function handleKeydown(event: KeyboardEvent) {
    // isComposing：中文输入法组词时按 Esc 只取消组词，不收起窗口
    if (event.key === "Escape" && !event.isComposing) {
      hide();
    }
  }

  let unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    window.addEventListener("keydown", handleKeydown);
    unlisteners = await Promise.all([
      onLauncherOpen(() => {
        query.value = "";
        options.onOpen?.();
      }),
      onLauncherClose(() => {
        query.value = "";
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
