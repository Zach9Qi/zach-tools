import { invoke } from "@tauri-apps/api/core";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauriRuntime } from "@/lib/runtime";

/** 隐藏启动器窗口（对应后端 hide_launcher 命令） */
export function hideLauncher(): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("hide_launcher");
}

/**
 * 把窗口高度贴到页面根元素高度（uTools 式自适应），宽度始终不变。
 * 宽度直接读当前视口：无边框且不可缩放，CSS 视口宽 = 窗口逻辑宽，
 * 唯一来源是 tauri.conf.json5 的 width，前端不再另存一份。
 * 高度上限不在这里维护：面板 max-h 封顶，由 CSS 决定，传入的测量值天然不会超过上限。
 */
export function resizeLauncherToContent(contentHeight: number): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return getCurrentWindow().setSize(new LogicalSize(window.innerWidth, Math.ceil(contentHeight)));
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
