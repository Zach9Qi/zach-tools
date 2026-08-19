import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** 隐藏启动器窗口（对应后端 hide_launcher 命令） */
export function hideLauncher(): Promise<void> {
  return invoke("hide_launcher");
}

/** 监听「启动器唤起」事件（后端通过全局快捷键展示窗口后触发） */
export function onLauncherOpen(handler: () => void): Promise<UnlistenFn> {
  return listen("launcher-open", () => handler());
}

/** 监听「启动器收起」事件（快捷键收起或窗口失焦隐藏后触发） */
export function onLauncherClose(handler: () => void): Promise<UnlistenFn> {
  return listen("launcher-close", () => handler());
}
