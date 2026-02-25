<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";

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
const menuStyle = ref<Record<string, string>>({});

const flattenedItems = computed(() => {
  return props.items.filter((item) => !item.divider);
});

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

  menuStyle.value = {
    position: "fixed",
    top: `${top}px`,
    left: `${left}px`,
    minWidth: props.minWidth,
    zIndex: "99999",
  };
};

const open = () => {
  isOpen.value = true;
  highlightedIndex.value = -1;
  nextTick(() => {
    updatePosition();
  });
};

const close = () => {
  isOpen.value = false;
  highlightedIndex.value = -1;
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
      highlightedIndex.value = Math.min(highlightedIndex.value + 1, flattenedItems.value.length - 1);
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

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("scroll", handleScroll, true);
  window.addEventListener("resize", handleScroll);
});

onUnmounted(() => {
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
          :style="menuStyle"
          role="menu"
          @keydown="handleKeydown"
        >
          <div class="sl-menu-content">
            <template v-for="(item, index) in items" :key="item.id">
              <div v-if="item.divider" class="sl-menu-divider" role="separator" />
              <div
                v-else
                class="sl-menu-item"
                :class="{
                  disabled: item.disabled,
                  danger: item.danger,
                  highlighted: flattenedItems.indexOf(item) === highlightedIndex,
                }"
                role="menuitem"
                :aria-disabled="item.disabled"
                @click="handleItemClick(item)"
                @mouseenter="highlightedIndex = flattenedItems.indexOf(item)"
              >
                <component v-if="item.icon" :is="item.icon" class="menu-icon" :size="16" />
                <span class="menu-label">{{ item.label }}</span>
              </div>
            </template>
          </div>
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
</style>

<style>
.sl-menu {
  background: var(--sl-surface, #ffffff);
  border: 1px solid var(--sl-border, #e2e8f0);
  border-radius: var(--sl-radius-lg, 12px);
  box-shadow:
    0 10px 25px rgba(0, 0, 0, 0.1),
    0 4px 10px rgba(0, 0, 0, 0.05);
  overflow: hidden;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  will-change: transform, opacity;
}

[data-theme="dark"] .sl-menu {
  background: var(--sl-surface, #1e2130);
  border-color: var(--sl-border, rgba(255, 255, 255, 0.1));
  box-shadow:
    0 10px 25px rgba(0, 0, 0, 0.3),
    0 4px 10px rgba(0, 0, 0, 0.2);
}

.sl-menu-content {
  padding: var(--sl-space-xs, 4px);
}

.sl-menu-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm, 8px);
  padding: 10px 12px;
  border-radius: var(--sl-radius-md, 8px);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
  user-select: none;
  position: relative;
  overflow: hidden;
}

.sl-menu-item::before {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--sl-primary, #0ea5e9);
  opacity: 0;
  transform: scale(0.5);
  transition: opacity 0.2s ease, transform 0.2s ease;
  border-radius: inherit;
}

.sl-menu-item:hover:not(.disabled),
.sl-menu-item.highlighted:not(.disabled) {
  background: var(--sl-surface-hover, #f1f5f9);
}

.sl-menu-item:active:not(.disabled)::before {
  opacity: 0.1;
  transform: scale(1);
}

[data-theme="dark"] .sl-menu-item:hover:not(.disabled),
[data-theme="dark"] .sl-menu-item.highlighted:not(.disabled) {
  background: var(--sl-surface-hover, rgba(255, 255, 255, 0.05));
}

.sl-menu-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sl-menu-item.danger {
  color: var(--sl-error, #ef4444);
}

.sl-menu-item.danger:hover:not(.disabled) {
  background: var(--sl-error-bg, rgba(239, 68, 68, 0.1));
}

.menu-icon {
  flex-shrink: 0;
  color: var(--sl-text-tertiary, #94a3b8);
}

.sl-menu-item.danger .menu-icon {
  color: var(--sl-error, #ef4444);
}

.menu-label {
  font-size: 0.875rem;
  color: var(--sl-text-primary, #0f172a);
  white-space: nowrap;
}

.sl-menu-item.danger .menu-label {
  color: var(--sl-error, #ef4444);
}

.sl-menu-divider {
  height: 1px;
  background: var(--sl-border, #e2e8f0);
  margin: var(--sl-space-xs, 4px) 0;
}

[data-theme="dark"] .sl-menu-divider {
  background: var(--sl-border, rgba(255, 255, 255, 0.1));
}

/* Animation */
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

/* Item stagger animation */
.sl-menu-item {
  animation: item-fade-in 0.2s ease backwards;
}

.sl-menu-item:nth-child(1) { animation-delay: 0.02s; }
.sl-menu-item:nth-child(2) { animation-delay: 0.04s; }
.sl-menu-item:nth-child(3) { animation-delay: 0.06s; }
.sl-menu-item:nth-child(4) { animation-delay: 0.08s; }
.sl-menu-item:nth-child(5) { animation-delay: 0.1s; }
.sl-menu-item:nth-child(6) { animation-delay: 0.12s; }
.sl-menu-item:nth-child(7) { animation-delay: 0.14s; }
.sl-menu-item:nth-child(8) { animation-delay: 0.16s; }

@keyframes item-fade-in {
  from {
    opacity: 0;
    transform: translateX(-8px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>
