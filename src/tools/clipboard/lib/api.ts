import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauriRuntime } from "@/lib/runtime";

/** 剪贴板内容类型,与后端 ClipboardKind 的序列化值一一对应 */
export type ClipboardKind = "text" | "image" | "files";

/** 剪贴板历史条目(预览形态),与后端 ClipboardItem(camelCase 序列化)一一对应 */
export interface ClipboardItem {
  /** 主键,操作(粘贴/复制/删除)时回传 */
  id: number;
  /** 内容类型;后端已入库 text 与 image,files 为预留 */
  kind: ClipboardKind;
  /** [text] 文本预览(最多 5000 字符);原文不出库,粘贴/复制按 id 在后端现取 */
  textPreview: string | null;
  /** [text] 原文总字符数,配合 textPreview 判断是否被截断 */
  textLength: number | null;
  /** [image] 原图落盘路径 */
  imagePath: string | null;
  /** [image] 列表缩略图落盘路径 */
  thumbnailPath: string | null;
  /** [image] 原图像素宽度 */
  imageWidth: number | null;
  /** [image] 原图像素高度 */
  imageHeight: number | null;
  /** [files] 文件/文件夹绝对路径列表 */
  filePaths: string[] | null;
  /** 是否收藏(收藏项不参与容量清理) */
  isFavorite: boolean;
  /** 首次记录时间(epoch 毫秒) */
  createdAt: number;
  /** 最近一次复制/使用时间(epoch 毫秒) */
  lastUsedAt: number;
}

/** keyset 分页游标:上一页最后一行的 (lastUsedAt, id)。值锚点,期间的插入/删除不影响翻页 */
export interface ClipboardListCursor {
  /** 最后一行的最近使用时间(epoch 毫秒) */
  lastUsedAt: number;
  /** 最后一行的 id,同毫秒时间戳的决胜键 */
  id: number;
}

/** 列表查询参数,均可省略;过滤维度(关键字/类型/收藏)可叠加 */
export interface ListClipboardParams {
  /** 关键字,对文本内容做包含匹配 */
  query?: string;
  /** 限定内容类型,缺省不限 */
  kind?: ClipboardKind;
  /** 只看收藏 */
  favoriteOnly?: boolean;
  /** 单页条数 */
  limit?: number;
  /** keyset 游标,缺省返回首页 */
  cursor?: ClipboardListCursor;
}

/**
 * 把本地绝对路径转成 WebView 可加载的 asset 协议 URL(用于 image 条目的缩略图 / 原图)。
 * 非 Tauri 运行时或路径为空时返回空串:浏览器预览下 `<img src="">` 只是不显示,不会报错。
 * 可访问范围由 tauri.conf.json5 的 assetProtocol.scope 管控,前端不做路径校验。
 */
export function toAssetUrl(path: string | null): string {
  if (!isTauriRuntime() || !path) {
    return "";
  }
  return convertFileSrc(path);
}

/** 分页查询历史(按最近使用倒序,预览形态) */
export function listClipboardItems(params: ListClipboardParams = {}): Promise<ClipboardItem[]> {
  if (!isTauriRuntime()) {
    return Promise.resolve([]);
  }
  return invoke("list_clipboard_items", { ...params });
}

/** 粘贴条目:写剪贴板 → 隐藏启动器 → 还原焦点到原应用 → 注入 Ctrl+V */
export function pasteClipboardItem(id: number): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("paste_clipboard_item", { id });
}

/** 仅把条目内容复制到系统剪贴板,面板保持打开 */
export function copyClipboardItem(id: number): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("copy_clipboard_item", { id });
}

/** 删除一条历史记录 */
export function deleteClipboardItem(id: number): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("delete_clipboard_item", { id });
}

/** 设置条目收藏状态;收藏项不参与容量清理 */
export function setClipboardFavorite(id: number, favorite: boolean): Promise<void> {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke("set_clipboard_favorite", { id, favorite });
}

/**
 * 监听新条目落库(载荷为预览形态,不携带原文)。
 * 注意:重复复制已有内容时,后端会以同一 id、刷新过 lastUsedAt 的条目重发,
 * 消费方需按 id 去重(已存在则上浮,而不是重复插入)。
 */
export function onClipboardNewItem(handler: (item: ClipboardItem) => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }
  return listen<ClipboardItem>("clipboard-new-item", (event) => handler(event.payload));
}
