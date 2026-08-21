/**
 * 输入形态判断辅助函数,给各工具的 accepts 复用。
 * 规则留在工具侧组合,不做成 ToolItem 上的枚举字段。
 */

/** 输入是否是可直接打开的 http(s) 链接 */
export function isUrl(query: string): boolean {
  const value = query.trim();
  if (!/^https?:\/\//i.test(value)) {
    return false;
  }
  try {
    new URL(value);
    return true;
  } catch {
    return false;
  }
}

/** 输入是否像本地文件路径(盘符开头或 UNC 路径) */
export function isPath(query: string): boolean {
  const value = query.trim();
  return /^[a-z]:[\\/]/i.test(value) || value.startsWith("\\\\");
}
