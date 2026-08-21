import { onMounted, onUnmounted, type Ref } from "vue";
import { resizeLauncherToContent } from "@/launcher/lib/window";

/**
 * 观察页面根元素(面板 + 阴影边距)的高度,变化时把窗口高度贴上去(uTools 式自适应)。
 * 高度上限完全由 CSS 决定:面板 max-h 封顶后根元素不再变高,窗口也随之封顶。
 */
export function useAutoHeight(root: Ref<HTMLElement | null>) {
  let observer: ResizeObserver | null = null;

  onMounted(() => {
    const element = root.value;
    if (!element) {
      return;
    }
    observer = new ResizeObserver(() => {
      void resizeLauncherToContent(element.offsetHeight);
    });
    observer.observe(element);
  });

  onUnmounted(() => {
    observer?.disconnect();
  });
}
