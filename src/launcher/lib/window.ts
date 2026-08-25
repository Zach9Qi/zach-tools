import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauriRuntime } from "@/lib/runtime";

/** 窗口固定宽度（逻辑像素，与 tauri.conf.json 的 width 一致） */
const WINDOW_WIDTH = 760;

/** 隐藏启动器窗口（对应后端 hide_launcher 命令） */
export function hideLauncher(): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("hide_launcher");
}

/**
 * 把窗口高度贴到页面根元素高度（uTools 式自适应），宽度始终不变。
 * 高度上限不在这里维护：面板 max-h 封顶、阴影边距由外层 p-5 提供，
 * 全部由 CSS 决定，传入的测量值天然不会超过上限。
 */
export function resizeLauncherToContent(contentHeight: number): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return getCurrentWindow().setSize(new LogicalSize(WINDOW_WIDTH, Math.ceil(contentHeight)));
}

/** 监听「启动器唤起」事件（后端通过全局快捷键展示窗口后触发） */
export function onLauncherOpen(handler: () => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }
  return listen("launcher-open", () => handler());
}

/** 监听「启动器收起」事件（快捷键收起或窗口失焦隐藏后触发） */
export function onLauncherClose(handler: () => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }
  return listen("launcher-close", () => handler());
}
