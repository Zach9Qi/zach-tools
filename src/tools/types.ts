import type { Component } from "vue";

/** 回车 / 点击磁贴时怎么打开 */
export type ToolAction =
  | {
      /** 同窗口切到该工具页,不隐藏启动器 */
      type: "view";
      /** 页面名,与 LauncherPanel 的 activeView 对应,如 clipboard */
      name: string;
    }
  | {
      /** 拉起外部程序或一次性动作,成功后由 run 自己隐藏启动器 */
      type: "launch";
      /** 执行动作;query 为触发时的搜索词(主页磁贴点入则为空串) */
      run: (ctx: { query: string }) => Promise<void>;
    };

/** 启动器工具目录中的一项 */
export interface ToolItem {
  /** 唯一标识,用法记录与打开时回传 */
  id: string;
  /** 磁贴主行名称,默认参与名称搜索 */
  title: string;
  /** lucide 图标名(如 clipboard),由 ToolTile 映射成组件,不在目录里存 Vue 组件 */
  icon: string;
  /** 名称搜索别名(剪切板、clipboard 等),不含 title */
  keywords: string[];
  /**
   * 当前输入能否作为该工具入参。
   * 不提供或恒返回 false = 不进匹配结果(设置、市场等)。
   */
  accepts?: (query: string) => boolean;
  /** 点击或回车时执行的动作 */
  action: ToolAction;
}

/** view 型工具的注册单元:只有这种形状允许携带工具页配置 */
export interface ViewToolModule {
  /** 进启动器目录与主页搜索;action 限定为 view */
  item: ToolItem & { action: Extract<ToolAction, { type: "view" }> };
  /** 同窗口工具页;对应里程碑未接入时可暂缺,registry 会跳过没有页面的 view 工具 */
  page?: Component;
  /** 进入该页时搜索框 placeholder */
  placeholder?: string;
}

/** launch 型工具的注册单元:只有目录项 */
export interface LaunchToolModule {
  /** 进启动器目录与主页搜索;action 限定为 launch */
  item: ToolItem & { action: Extract<ToolAction, { type: "launch" }> };
  /** launch 型没有工具页,never 让误配在编译期报错 */
  page?: never;
  /** placeholder 只对工具页有意义,同样禁止 */
  placeholder?: never;
}

/** 一个内置工具的注册单元;按 action.type 拆成两种形状,「view 型才有 page」由编译器保证 */
export type ToolModule = ViewToolModule | LaunchToolModule;

/** 空查询主页;recent / pinned 是目录项的引用结果,不是另一套类型 */
export interface HomeResults {
  /** 最近使用,最多 8 个,useResults 已裁掉多余项 */
  recent: ToolItem[];
  /** 已固定,最多两行共 16 个,再多的进「全部」;useResults 已裁切 */
  pinned: ToolItem[];
}
