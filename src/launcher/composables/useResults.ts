import { computed, watch, type Ref } from "vue";
import { useRowNavigation } from "@/launcher/composables/useRowNavigation";
import { catalog } from "@/tools/registry";
import type { HomeResults, ToolItem } from "@/tools/types";

/** useResults 配置 */
export interface UseResultsOptions {
  /** 当前搜索词(来自外壳 useLauncher) */
  query: Ref<string>;
  /** 回车 / 点击磁贴时回调,由调用方决定 view / launch 怎么走 */
  onActivate: (tool: ToolItem) => void;
}

/** 结果区磁贴分区里的一个条目:工具 + 它在展平顺序中的全局下标 */
export interface SectionEntry {
  /** 目录项 */
  tool: ToolItem;
  /** 展平后的全局下标,sections 派生时盖章;组件只做相等比较,不做坐标换算 */
  index: number;
}

/** 结果区的一个磁贴分区;模板渲染与导航展平共用这份描述,顺序与下标不会错位 */
export interface ResultSection {
  /** 分区标识,模板 key 与分区特有 UI(如已固定的「全部」)按它分支 */
  key: "recent" | "pinned" | "matches" | "named";
  /** 分区标题 */
  title: string;
  /** 本区条目(含全局下标) */
  entries: SectionEntry[];
}

/** 一行磁贴的容量,窗口宽度按一行 8 个定死 */
const ROW_CAPACITY = 8;

/**
 * 主页:用法存储尚未接入,最近使用先展示真实目录(当前只有剪贴板),已固定留空。
 * 接入后改从存储读 id 再解析成目录项,这里只负责按容量裁切。
 */
const home: HomeResults = {
  recent: catalog.slice(0, ROW_CAPACITY),
  pinned: [],
};

/** 按一行容量切成屏幕行,↑↓ 按这些行跳 */
function chunkRows(items: ToolItem[]): ToolItem[][] {
  const rows: ToolItem[][] = [];
  for (let i = 0; i < items.length; i += ROW_CAPACITY) {
    rows.push(items.slice(i, i + ROW_CAPACITY));
  }
  return rows;
}

interface SectionDef {
  key: ResultSection["key"];
  title: string;
  items: ToolItem[];
}


function stampSections(defs: SectionDef[]): ResultSection[] {
  let offset = 0;
  return defs
    .filter((def) => def.items.length > 0)
    .map((def) => {
      // offset 记着前面的分区已经编到几号;本区第一个磁贴从 start 起,第 i 个就是 start + i
      const start = offset;
      offset += def.items.length;
      return {
        key: def.key,
        title: def.title,
        entries: def.items.map((tool, i) => ({ tool, index: start + i })),
      };
    });
}

/**
 * 结果区数据编排:主页(最近使用 / 已固定)与搜索态(匹配结果 / 搜索结果)两套分区,
 * sections(模板渲染)与 rows(导航展平)从同一份分区定义派生,不会互相错位;
 * 选中与键盘交给 useRowNavigation,这里只决定分区怎么组、何时复位选中。
 */
export function useResults({ query, onActivate }: UseResultsOptions) {
  /** query 非空即搜索态,与 ResultsPanel 的两套编排一一对应 */
  const isSearch = computed(() => query.value !== "");

  /** 匹配结果:当前输入能直接当作工具入参 */
  const matches = computed(() => catalog.filter((tool) => tool.accepts?.(query.value)));

  /** 搜索结果:工具名 / 关键字字面命中;已进匹配结果的不重复出现 */
  const named = computed(() => {
    const keyword = query.value.trim().toLowerCase();
    if (!keyword) {
      return [];
    }
    return catalog.filter(
      (tool) =>
        !matches.value.includes(tool) &&
        (tool.title.toLowerCase().includes(keyword) ||
          tool.keywords.some((alias) => alias.toLowerCase().includes(keyword))),
    );
  });

  /** 当前状态下按屏幕顺序排列的分区,已剔除空分区、盖好全局下标 */
  const sections = computed<ResultSection[]>(() =>
    isSearch.value
      ? stampSections([
          { key: "matches", title: "匹配结果", items: matches.value },
          { key: "named", title: "搜索结果", items: named.value },
        ])
      : stampSections([
          { key: "recent", title: "最近使用", items: home.recent },
          { key: "pinned", title: "已固定", items: home.pinned },
        ]),
  );

  /**
   * 按屏幕行分组的可选项:每行最多 8 个磁贴,分区超过一行时拆开。
   * ↑↓ 在这些行之间跳;剪贴板列表接入后,每个列表项作为单独一行插到最前。
   */
  const rows = computed<ToolItem[][]>(() =>
    sections.value.flatMap((section) => chunkRows(section.entries.map((entry) => entry.tool))),
  );

  const { selectedIndex, select, reset } = useRowNavigation({ rows, onActivate });

  // 查询变化时选中归零(主页 ↔ 搜索态切换也走这里);
  // 窗口收起再唤起不复位,选中与搜索词一起保留上次状态
  watch(query, () => reset());

  return { isSearch, sections, selectedIndex, select };
}
