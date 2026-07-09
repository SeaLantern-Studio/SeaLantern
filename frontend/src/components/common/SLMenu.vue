<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from "vue";
import SLMenuBase from "./SLMenuBase.vue";

interface MenuItem {
  id: string | number;
  label: string;
  icon?: any;
  disabled?: boolean;
  danger?: boolean;
  divider?: boolean;
  children?: MenuItem[];
}

interface Props {
  items: MenuItem[];
  position?: "bottom-start" | "bottom-end" | "top-start" | "top-end";
  offset?: number;
  minWidth?: string;
}

const props = withDefaults(defineProps<Props>(), {
  position: "bottom-start",
  offset: 4,
  minWidth: "160px",
});

const emit = defineEmits<{
  select: [item: MenuItem];
}>();

const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const highlightedIndex = ref(-1);
const menuStyle = ref<Record<string, string>>({
  position: "fixed",
  top: "0px",
  left: "0px",
  zIndex: "99999",
  visibility: "hidden",
  pointerEvents: "none",
});
const isPositioned = ref(false);
let positionFrameId: number | null = null;

const flattenedItems = computed(() => {
  return props.items.filter((item) => !item.divider);
});

const resetMenuStyle = () => {
  menuStyle.value = {
    position: "fixed",
    top: "0px",
    left: "0px",
    zIndex: "99999",
    visibility: "hidden",
    pointerEvents: "none",
  };
};

const cancelScheduledPositioning = () => {
  if (positionFrameId !== null) {
    cancelAnimationFrame(positionFrameId);
    positionFrameId = null;
  }
};

const updatePosition = () => {
  if (!triggerRef.value || !menuRef.value) return;

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const menuRect = menuRef.value.getBoundingClientRect();
  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;

  let top = 0;
  let left = 0;

  switch (props.position) {
    case "bottom-start":
      top = triggerRect.bottom + props.offset;
      left = triggerRect.left;
      break;
    case "bottom-end":
      top = triggerRect.bottom + props.offset;
      left = triggerRect.right - menuRect.width;
      break;
    case "top-start":
      top = triggerRect.top - menuRect.height - props.offset;
      left = triggerRect.left;
      break;
    case "top-end":
      top = triggerRect.top - menuRect.height - props.offset;
      left = triggerRect.right - menuRect.width;
      break;
  }

  if (left < 0) left = 8;
  if (left + menuRect.width > viewportWidth) left = viewportWidth - menuRect.width - 8;
  if (top < 0) top = triggerRect.bottom + props.offset;
  if (top + menuRect.height > viewportHeight) top = viewportHeight - menuRect.height - 8;

  if (top < 8) top = 8;
  if (left < 8) left = 8;

  menuStyle.value = {
    position: "fixed",
    top: `${top}px`,
    left: `${left}px`,
    zIndex: "99999",
    visibility: "visible",
    pointerEvents: "auto",
  };
  isPositioned.value = true;
};

const schedulePositionUpdate = () => {
  cancelScheduledPositioning();
  nextTick(() => {
    if (!isOpen.value) return;
    updatePosition();
    positionFrameId = requestAnimationFrame(() => {
      positionFrameId = null;
      if (!isOpen.value) return;
      updatePosition();
    });
  });
};

const open = () => {
  isOpen.value = true;
  highlightedIndex.value = -1;
  isPositioned.value = false;
  resetMenuStyle();
  schedulePositionUpdate();
};

const close = () => {
  isOpen.value = false;
  highlightedIndex.value = -1;
  isPositioned.value = false;
  cancelScheduledPositioning();
  resetMenuStyle();
};

const toggle = () => {
  if (isOpen.value) {
    close();
  } else {
    open();
  }
};

const handleItemClick = (item: MenuItem) => {
  if (item.disabled) return;
  emit("select", item);
  close();
};

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) {
    if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
      e.preventDefault();
      open();
    }
    return;
  }

  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      highlightedIndex.value = Math.min(
        highlightedIndex.value + 1,
        flattenedItems.value.length - 1,
      );
      break;
    case "ArrowUp":
      e.preventDefault();
      highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0);
      break;
    case "Enter":
    case " ":
      e.preventDefault();
      if (highlightedIndex.value >= 0) {
        handleItemClick(flattenedItems.value[highlightedIndex.value]);
      }
      break;
    case "Escape":
      e.preventDefault();
      close();
      break;
  }
};

const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as Node;
  if (
    triggerRef.value &&
    !triggerRef.value.contains(target) &&
    menuRef.value &&
    !menuRef.value.contains(target)
  ) {
    close();
  }
};

const handleScroll = () => {
  if (isOpen.value) {
    updatePosition();
  }
};

watch(
  () => props.items,
  () => {
    if (isOpen.value) {
      schedulePositionUpdate();
    }
  },
  { deep: true },
);

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("scroll", handleScroll, true);
  window.addEventListener("resize", handleScroll);
});

onUnmounted(() => {
  cancelScheduledPositioning();
  document.removeEventListener("click", handleClickOutside);
  window.removeEventListener("scroll", handleScroll, true);
  window.removeEventListener("resize", handleScroll);
});

defineExpose({ open, close, toggle });
</script>

<template>
  <div class="sl-menu-wrapper">
    <div
      ref="triggerRef"
      class="sl-menu-trigger"
      @click="toggle"
      @keydown="handleKeydown"
      tabindex="0"
      role="button"
      :aria-expanded="isOpen"
      :aria-haspopup="true"
    >
      <slot />
    </div>

    <Teleport to="body">
      <Transition name="menu">
        <div
          v-if="isOpen"
          ref="menuRef"
          class="sl-menu"
          :class="{ 'sl-menu-positioning': !isPositioned }"
          :style="menuStyle"
          role="menu"
          @keydown="handleKeydown"
        >
          <SLMenuBase :items="items" :min-width="minWidth" @select="handleItemClick" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.sl-menu-wrapper {
  display: inline-flex;
}

.sl-menu-trigger {
  display: inline-flex;
  cursor: pointer;
  outline: none;
  border-radius: var(--sl-radius-sm, 6px);
  transition: background-color 0.15s ease;
}

.sl-menu-trigger:focus-visible {
  box-shadow: 0 0 0 2px var(--sl-primary, #0ea5e9);
}

.sl-menu {
  animation: menu-enter 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sl-menu-positioning {
  opacity: 0;
}

.menu-enter-active {
  animation: menu-enter 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.menu-leave-active {
  animation: menu-leave 0.15s ease;
}

@keyframes menu-enter {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes menu-leave {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
}
</style>
