import { ref, watch, onMounted, nextTick, type Ref } from "vue";

/**
 * Tab 指示器位置
 */
export interface TabIndicatorPosition {
  left: number;
  width: number;
}

/**
 * Tab 指示器动画 composable
 * 用于管理 Tab 切换时的滑动指示器动画
 */
export function useTabIndicator<T extends string>(
  activeTab: Ref<T>,
  options: {
    /** 延迟更新时间（毫秒） */
    delay?: number;
    /** 容器选择器 */
    containerSelector?: string;
    /** 激活状态的 CSS 类名 */
    activeClass?: string;
  } = {},
) {
  const { delay = 50, activeClass = "active" } = options;

  const indicatorRef = ref<HTMLElement | null>(null);
  const position = ref<TabIndicatorPosition>({ left: 0, width: 0 });

  /**
   * 更新指示器位置
   */
  function updatePosition(): void {
    nextTick(() => {
      if (!indicatorRef.value) return;

      const container = indicatorRef.value.parentElement;
      if (!container) return;

      const activeTabBtn = container.querySelector(`.${activeClass}`);
      if (activeTabBtn) {
        const { offsetLeft, offsetWidth } = activeTabBtn as HTMLElement;
        position.value = {
          left: offsetLeft,
          width: offsetWidth,
        };

        indicatorRef.value.style.left = `${offsetLeft}px`;
        indicatorRef.value.style.width = `${offsetWidth}px`;
      }
    });
  }

  /**
   * 延迟更新指示器位置
   */
  function delayedUpdate(): void {
    if (delay > 0) {
      setTimeout(updatePosition, delay);
    } else {
      updatePosition();
    }
  }

  /**
   * 手动设置指示器位置
   */
  function setPosition(left: number, width: number): void {
    position.value = { left, width };
    if (indicatorRef.value) {
      indicatorRef.value.style.left = `${left}px`;
      indicatorRef.value.style.width = `${width}px`;
    }
  }

  /**
   * 重置指示器位置
   */
  function reset(): void {
    position.value = { left: 0, width: 0 };
    if (indicatorRef.value) {
      indicatorRef.value.style.left = "0px";
      indicatorRef.value.style.width = "0px";
    }
  }

  // 监听 activeTab 变化
  watch(activeTab, () => {
    delayedUpdate();
  });

  // 组件挂载后初始化
  onMounted(() => {
    delayedUpdate();
  });

  return {
    indicatorRef,
    position,
    updatePosition,
    delayedUpdate,
    setPosition,
    reset,
  };
}

/**
 * 简化版 Tab 指示器
 * 直接返回模板中需要绑定的 ref 和样式
 */
export function useSimpleTabIndicator() {
  const indicatorRef = ref<HTMLElement | null>(null);

  /**
   * 更新指示器位置
   */
  function updatePosition(): void {
    nextTick(() => {
      if (!indicatorRef.value) return;

      const container = indicatorRef.value.parentElement;
      if (!container) return;

      const activeTabBtn = container.querySelector(".active");
      if (activeTabBtn) {
        const { offsetLeft, offsetWidth } = activeTabBtn as HTMLElement;
        indicatorRef.value.style.left = `${offsetLeft}px`;
        indicatorRef.value.style.width = `${offsetWidth}px`;
      }
    });
  }

  return {
    indicatorRef,
    updatePosition,
  };
}

/**
 * Tab 切换管理 composable
 * 结合 activeTab 和指示器动画
 */
export function useTabSwitch<T extends string>(
  initialTab: T,
  options: {
    delay?: number;
    activeClass?: string;
    onChange?: (newTab: T, oldTab: T) => void;
  } = {},
) {
  const { delay = 50, activeClass = "active", onChange } = options;

  const activeTab = ref<T>(initialTab) as Ref<T>;
  const indicatorRef = ref<HTMLElement | null>(null);

  /**
   * 切换到指定 Tab
   */
  function switchTab(tab: T): void {
    if (activeTab.value === tab) return;

    const oldTab = activeTab.value;
    activeTab.value = tab;
    updateIndicator();

    onChange?.(tab, oldTab);
  }

  /**
   * 更新指示器位置
   */
  function updateIndicator(): void {
    setTimeout(() => {
      nextTick(() => {
        if (!indicatorRef.value) return;

        const container = indicatorRef.value.parentElement;
        if (!container) return;

        const activeTabBtn = container.querySelector(`.${activeClass}`);
        if (activeTabBtn) {
          const { offsetLeft, offsetWidth } = activeTabBtn as HTMLElement;
          indicatorRef.value.style.left = `${offsetLeft}px`;
          indicatorRef.value.style.width = `${offsetWidth}px`;
        }
      });
    }, delay);
  }

  onMounted(() => {
    updateIndicator();
  });

  return {
    activeTab,
    indicatorRef,
    switchTab,
    updateIndicator,
  };
}
