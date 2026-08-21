import type { ToolModule } from "@/tools/types";

/**
 * 剪贴板工具:当前里程碑只注册目录项(磁贴可见、参与匹配),
 * 工具页(page: ClipboardPage)与列表数据等剪贴板页里程碑再接。
 */
export const clipboardTool: ToolModule = {
  item: {
    id: "clipboard",
    title: "剪贴板",
    icon: "clipboard",
    keywords: ["剪切板", "剪贴板历史", "clipboard", "粘贴"],
    // 任意非空文本都能进剪贴板搜索,因此有字就进匹配结果
    accepts: (query) => query.trim().length > 0,
    action: { type: "view", name: "clipboard" },
  },
  placeholder: "搜索剪贴板",
};
