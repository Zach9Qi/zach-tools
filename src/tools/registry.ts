import type { Component } from "vue";
import { clipboardTool } from "@/tools/clipboard";
import type { ToolItem, ToolModule } from "@/tools/types";

/** 已注册的内置工具;将来插件系统只换这里的注册来源 */
const modules: ToolModule[] = [clipboardTool];

/** 启动器工具目录,参与主页展示与搜索 / 匹配 */
export const catalog: ToolItem[] = modules.map((m) => m.item);

/** view 名 → 工具页组件,LauncherPanel 据此切页;工具未提供 page 则不进表 */
export const toolPages: Record<string, Component> = Object.fromEntries(
  modules.flatMap((m) =>
    m.page && m.item.action.type === "view" ? [[m.item.action.name, m.page]] : [],
  ),
);
