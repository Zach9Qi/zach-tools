import { computed, ref, type Ref } from "vue";
import { useKeymap } from "@/launcher/composables/useKeymap";

/** useRowNavigation 配置 */
export interface UseRowNavigationOptions<T> {
  /** 按屏幕行分组的可选项;行怎么切由调用方决定(磁贴分区按容量切、列表每项一行) */
  rows: Ref<T[][]>;
  /** Enter 触发当前选中项 */
  onActivate: (item: T) => void;
}

/**
 * 行网格导航机:只认「按行分组的可选项」,不关心条目是磁贴还是列表行。
 * 全部条目从上到下、从左到右展平成一维 selectedIndex:
 * ←→ 沿展平顺序 ±1(行尾顺到下一行),↑↓ 行间跳转且列号就近保留,
 * 两个方向都首尾回绕,Enter 触发选中项。何时复位选中由调用方通过 reset 决定。
 * 按键经 useKeymap 登记,页脚提示由同一份定义派生。
 */
export function useRowNavigation<T>({ rows, onActivate }: UseRowNavigationOptions<T>) {
  /** 当前全部可选项的展平顺序,selectedIndex 指向这里 */
  const flat = computed<T[]>(() => rows.value.flat());

  const selectedIndex = ref(0);

  /** 鼠标悬停等场景直接指定选中项(展平后的下标) */
  function select(index: number) {
    selectedIndex.value = index;
  }

  /** 回到默认选中第一项;复位时机(如查询变化)由调用方决定 */
  function reset() {
    selectedIndex.value = 0;
  }

  /** 展平下标 → 行列坐标;越界返回 null */
  function locate(index: number): { row: number; col: number } | null {
    let offset = 0;
    for (const [row, items] of rows.value.entries()) {
      if (index < offset + items.length) {
        return { row, col: index - offset };
      }
      offset += items.length;
    }
    return null;
  }

  /** 行列坐标 → 展平下标(调用方保证行存在、列不越界) */
  function indexAt(row: number, col: number): number {
    let offset = 0;
    for (let r = 0; r < row; r += 1) {
      offset += rows.value[r].length;
    }
    return offset + col;
  }

  /** ←→:沿展平顺序 ±1,行尾继续按会顺到下一行,末尾与开头首尾回绕 */
  function moveFlat(delta: 1 | -1) {
    const count = flat.value.length;
    if (count === 0) {
      return;
    }
    selectedIndex.value = (selectedIndex.value + delta + count) % count;
  }

  /** ↑↓:跳到相邻行,列号就近保留(目标行更短时贴到行尾),顶行与底行之间回绕 */
  function moveRow(delta: 1 | -1) {
    const position = locate(selectedIndex.value);
    if (!position) {
      return;
    }
    const count = rows.value.length;
    const targetRow = (position.row + delta + count) % count;
    const target = rows.value[targetRow];
    selectedIndex.value = indexAt(targetRow, Math.min(position.col, target.length - 1));
  }

  // 按键、页脚文案与回调同源登记;文案先写死,出现第二个使用方再提成选项。
  // 行导航优先于输入框光标移动(启动器场景改词以退格为主),接管由 useKeymap 统一 preventDefault
  useKeymap([
    {
      keys: ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"],
      label: "选择",
      onPress: (event) => {
        if (event.key === "ArrowUp" || event.key === "ArrowDown") {
          moveRow(event.key === "ArrowDown" ? 1 : -1);
        } else {
          moveFlat(event.key === "ArrowRight" ? 1 : -1);
        }
      },
    },
    {
      keys: ["Enter"],
      label: "打开",
      onPress: () => {
        const item = flat.value[selectedIndex.value];
        if (item) {
          onActivate(item);
        }
      },
    },
  ]);

  return { selectedIndex, select, reset };
}
