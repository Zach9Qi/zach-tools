import ClipboardPage from "@/tools/clipboard/components/ClipboardPage.vue";
import type { ViewToolModule } from "@/tools/types";

/** 剪贴板工具:目录项(磁贴、参与匹配)+ 同窗口工具页 */
export const clipboardTool: ViewToolModule = {
  item: {
    id: "clipboard",
    title: "剪贴板",
    icon: "clipboard",
    keywords: ["剪切板", "剪贴板历史", "clipboard", "粘贴"],
    // 任意非空文本都能进剪贴板搜索,因此有字就进匹配结果(名称命中时归搜索结果)
    accepts: (query) => query.trim().length > 0,
    action: "view",
  },
  page: ClipboardPage,
  placeholder: "搜索剪贴板",
};
