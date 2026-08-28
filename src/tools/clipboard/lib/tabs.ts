import type { ClipboardKind } from "@/tools/clipboard/lib/api";

/** 类型过滤 tab 定义:同时驱动 tab 行渲染与列表查询的 kind 参数 */
export interface ClipboardKindTab {
  /** 标识,tab 行渲染 key 与选中态比对 */
  key: string;
  /** tab 显示名 */
  label: string;
  /** 限定内容类型;缺省不限 */
  kind?: ClipboardKind;
}

/**
 * 类型 tab 注册表(单选)。「只看收藏」是独立的正交开关,不在此表,
 * 类型、收藏、搜索词三个维度叠加过滤。
 * 图片 / 文件监听入库(二期)后在此追加
 * { key: "image", label: "图片", kind: "image" } 等,tab 行渲染与 Tab 键轮切自动生效。
 */
export const KIND_TABS: ClipboardKindTab[] = [{ key: "all", label: "全部" }];
