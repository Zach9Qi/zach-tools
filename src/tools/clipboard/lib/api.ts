import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauriRuntime } from "@/lib/runtime";

/** 剪贴板内容类型,与后端 ClipboardKind 的序列化值一一对应 */
export type ClipboardKind = "text" | "image" | "files";

/** 剪贴板历史条目,与后端 ClipboardItem(camelCase 序列化)一一对应 */
export interface ClipboardItem {
  /** 主键,操作(粘贴/复制/删除)时回传 */
  id: number;
  /** 内容类型;当前后端只入库 text,image / files 为预留 */
  kind: ClipboardKind;
  /** [text] 文本内容 */
  textContent: string | null;
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

/** 列表查询参数,均可省略(后端默认 limit 100、上限 500) */
export interface ListClipboardParams {
  /** 关键字,对文本内容做包含匹配 */
  query?: string;
  /** 单页条数 */
  limit?: number;
  /** 跳过条数 */
  offset?: number;
}

/** 分页查询历史(按最近使用倒序) */
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

/**
 * 监听新条目落库。
 * 注意:重复复制已有内容时,后端会以同一 id、刷新过 lastUsedAt 的条目重发,
 * 消费方需按 id 去重(已存在则上浮,而不是重复插入)。
 */
export function onClipboardNewItem(handler: (item: ClipboardItem) => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(() => undefined);
  }
  return listen<ClipboardItem>("clipboard-new-item", (event) => handler(event.payload));
}
