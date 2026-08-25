import type { Component } from "vue";
import IconClipboard from "~icons/lucide/clipboard";
import IconPuzzle from "~icons/lucide/puzzle";

/** 目录里的 icon 字符串 → lucide 组件;新工具图标在这里登记 */
const icons: Record<string, Component> = {
  clipboard: IconClipboard,
};

/** 查目录图标,未登记的走拼图占位 */
export function iconOf(name: string): Component {
  return icons[name] ?? IconPuzzle;
}
