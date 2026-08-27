import { computed, onUnmounted, ref } from "vue";

/** 一条快捷键定义:按键、页脚文案与回调绑定在同一处,事件分发与页脚展示都由它派生 */
export interface KeyBinding {
  /** 绑定的按键(KeyboardEvent.key 值,如 "ArrowUp" / "Enter" / "Delete"),页脚键帽由此推导 */
  keys: string[];
  /** 页脚上的动作说明(如 "选择"、"粘贴") */
  label: string;
  /** 命中任一按键时的回调;一条绑定多个键需要分流时读 event.key */
  onPress: (event: KeyboardEvent) => void;
}

/** 页脚一条提示的展示形态:键帽标签 + 动作说明,由 KeyBinding 推导,不单独维护 */
export interface FooterHint {
  /** 键帽显示标签(如 ["↑", "↓"]) */
  keys: string[];
  /** 动作说明文案 */
  label: string;
}

/** KeyboardEvent.key → 键帽显示;不在表内的按键原样显示 */
const KEY_LABELS: Record<string, string> = {
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Enter: "↵",
  Delete: "Del",
  Escape: "Esc",
};

/** 当前挂载页面登记的快捷键表;null = 无登记 */
const registered = ref<KeyBinding[] | null>(null);

/**
 * 当前页面快捷键的登记处:一条定义同时驱动按键分发与页脚提示,结构上杜绝两边漂移。
 * 页面(主页结果区 / 工具页)在自己的 composable 里传 bindings 调用:
 * 挂载期间监听 window keydown 并分发到命中绑定的 onPress,卸载自动移除;
 * LauncherFooter 无参调用,只读派生出的提示列表。
 * Esc(返回/隐藏)归外壳,不在这里登记,页脚自行追加。
 */
export function useKeymap(bindings?: KeyBinding[]) {
  if (bindings) {
    registered.value = bindings;

    const handleKeydown = (event: KeyboardEvent) => {
      // 输入法组词与修饰键组合(如全局快捷键 Alt+Enter、Shift+←→ 选文本)不当作页面快捷键
      if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
        return;
      }
      const binding = bindings.find((b) => b.keys.includes(event.key));
      if (binding) {
        // 绑定即接管:挡掉输入框的默认行为(光标跳行首尾、前向删字);未绑定的键正常落入输入框
        event.preventDefault();
        binding.onPress(event);
      }
    };
    window.addEventListener("keydown", handleKeydown);

    onUnmounted(() => {
      window.removeEventListener("keydown", handleKeydown);
      // unmounted 钩子延迟到 patch 之后执行,页面切换时新页可能已登记,只清除仍属于自己的那份
      if (registered.value === bindings) {
        registered.value = null;
      }
    });
  }

  /** 页脚提示:按登记顺序把按键换成键帽标签 */
  const hints = computed<FooterHint[]>(() =>
    (registered.value ?? []).map((binding) => ({
      keys: binding.keys.map((key) => KEY_LABELS[key] ?? key),
      label: binding.label,
    })),
  );

  return { hints };
}
