import { clipboardTool } from "@/tools/clipboard";
import type { ToolItem, ToolModule } from "@/tools/types";

/** 已注册的工具;新增工具在 tools/<id>/ 实现 ToolModule 后加进这个数组即可 */
const modules: ToolModule[] = [clipboardTool];

/** 启动器工具目录,参与主页展示与搜索 / 匹配 */
export const catalog: ToolItem[] = modules.map((m) => m.item);

/** id → 注册单元,激活时据此找到工具的实现(page / run) */
const moduleById = new Map(modules.map((m) => [m.item.id, m]));

/** 按目录项查所属注册单元;目录项都源自 modules,查不到属于编程错误 */
export function moduleOf(item: ToolItem): ToolModule {
  const module = moduleById.get(item.id);
  if (!module) {
    throw new Error(`工具 ${item.id} 未注册`);
  }
  return module;
}
