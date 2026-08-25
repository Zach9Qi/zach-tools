import type { Component } from "vue";

/** 激活方式:view = 同窗口切到工具页,不隐藏启动器;launch = 执行一次性动作 */
export type ToolActionKind = "view" | "launch";

/**
 * 启动器工具目录中的一项:目录信息 + 匹配规则,搜索阶段的全部依赖。
 * 激活阶段才需要的实现(page / run)在所属 ToolModule 上,经 registry 按 id 查得。
 */
export interface ToolItem {
  /** 唯一标识,用法记录、注册表查找与打开时回传 */
  id: string;
  /** 磁贴主行名称,默认参与名称搜索 */
  title: string;
  /** lucide 图标名(如 clipboard),由 tools/icons 映射成组件,不在目录里存 Vue 组件 */
  icon: string;
  /** 名称搜索别名(剪切板、clipboard 等),不含 title;与 accepts 同为匹配规则,前者管名称命中 */
  keywords: string[];
  /**
   * 当前输入能否作为该工具入参(进「匹配结果」分区)。
   * 不提供或恒返回 false = 不进匹配结果(设置、市场等)。
   * 直接写函数,可用 tools/match 的辅助规则组合。
   */
  accepts?: (query: string) => boolean;
  /** 激活方式 */
  action: ToolActionKind;
}

/** view 型工具的注册单元:目录项 + 同窗口工具页 */
export interface ViewToolModule {
  /** 进启动器目录与主页搜索;action 限定为 view */
  item: ToolItem & { action: "view" };
  /**
   * 同窗口工具页,view 型必有页面(编译期保证)。
   * 页面组件接收 `query: string` prop(搜索框当前内容,作页内过滤词)。
   */
  page: Component;
  /** 进入该页时搜索框 placeholder */
  placeholder?: string;
  /** view 型没有一次性动作,never 让误配在编译期报错 */
  run?: never;
}

/** launch 型工具的注册单元:目录项 + 一次性动作 */
export interface LaunchToolModule {
  /** 进启动器目录与主页搜索;action 限定为 launch */
  item: ToolItem & { action: "launch" };
  /** 执行动作;query 为触发时的搜索词(主页磁贴点入则为空串),成功后由 run 自己隐藏启动器 */
  run: (ctx: { query: string }) => Promise<void>;
  /** launch 型没有工具页,never 让误配在编译期报错 */
  page?: never;
  /** placeholder 只对工具页有意义,同样禁止 */
  placeholder?: never;
}

/** 一个内置工具的注册单元;按 item.action 拆成两种形状,「view 型才有 page、launch 型才有 run」由编译器保证 */
export type ToolModule = ViewToolModule | LaunchToolModule;

/** ToolModule 联合的收窄谓词(嵌套判别字段 TS 不会自动收窄,统一走这里) */
export function isViewModule(module: ToolModule): module is ViewToolModule {
  return module.item.action === "view";
}

/** 空查询主页;recent / pinned 是目录项的引用结果,不是另一套类型 */
export interface HomeResults {
  /** 最近使用,最多 8 个,useResults 已裁掉多余项 */
  recent: ToolItem[];
  /** 已固定,最多两行共 16 个,再多的进「全部」;useResults 已裁切 */
  pinned: ToolItem[];
}
