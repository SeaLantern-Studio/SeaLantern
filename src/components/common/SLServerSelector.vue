<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Server, Check, ChevronDown } from "lucide-vue-next";
import { i18n } from "@language";

interface ServerOption {
  label: string;
  value: string;
}

interface Props {
  modelValue?: string;
  options: ServerOption[];
  collapsed?: boolean;
  placeholder?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: () => i18n.t("common.select_server"),
  collapsed: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const dropdownRef = ref<HTMLElement | null>(null);
const highlightedIndex = ref(-1);
const dropdownStyle = ref<Record<string, string>>({});

const selectedOption = computed(() => {
  if (!props.modelValue) return undefined;
  return props.options.find((opt) => opt.value === props.modelValue);
});

const displayLabel = computed(() => {
  return selectedOption.value?.label || props.placeholder;
});

const updateDropdownPosition = () => {
  if (!triggerRef.value) return;

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const viewportHeight = window.innerHeight;
  const viewportWidth = window.innerWidth;
  const dropdownWidth = 200;
  const dropdownMaxHeight = 300;
  const gap = 8;

  let top: number;
  let left: number;

  if (props.collapsed) {
    left = triggerRect.right + gap;
    top = triggerRect.top;
  } else {
    left = triggerRect.left;
    top = triggerRect.bottom + gap;
  }

  if (left + dropdownWidth > viewportWidth - gap) {
    left = viewportWidth - dropdownWidth - gap;
  }
  if (left < gap) {
    left = gap;
  }

  if (top + dropdownMaxHeight > viewportHeight - gap) {
    top = viewportHeight - dropdownMaxHeight - gap;
  }
  if (top < gap) {
    top = gap;
  }

  dropdownStyle.value = {
    position: "fixed",
    top: `${top}px`,
    left: `${left}px`,
    width: `${dropdownWidth}px`,
    maxHeight: `${dropdownMaxHeight}px`,
    zIndex: "99999",
  };
};

const toggleDropdown = () => {
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    highlightedIndex.value = -1;
    nextTick(() => {
      updateDropdownPosition();
    });
  }
};

const selectOption = (option: ServerOption) => {
  emit("update:modelValue", option.value);
  isOpen.value = false;
};

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) {
    if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      toggleDropdown();
    }
    return;
  }

  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      highlightedIndex.value = Math.min(highlightedIndex.value + 1, props.options.length - 1);
      scrollToHighlighted();
      break;
    case "ArrowUp":
      e.preventDefault();
      highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0);
      scrollToHighlighted();
      break;
    case "Enter":
    case " ":
      e.preventDefault();
      if (highlightedIndex.value >= 0 && props.options[highlightedIndex.value]) {
        selectOption(props.options[highlightedIndex.value]);
      }
      break;
    case "Escape":
      isOpen.value = false;
      break;
  }
};

const scrollToHighlighted = () => {
  requestAnimationFrame(() => {
    const highlighted = dropdownRef.value?.querySelector(".highlighted");
    highlighted?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });
};

const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as Node;
  if (
    triggerRef.value &&
    !triggerRef.value.contains(target) &&
    dropdownRef.value &&
    !dropdownRef.value.contains(target)
  ) {
    isOpen.value = false;
  }
};

const handleScroll = () => {
  if (isOpen.value) {
    updateDropdownPosition();
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

watch(
  () => props.collapsed,
  () => {
    if (isOpen.value) {
      nextTick(() => {
        updateDropdownPosition();
      });
    }
  },
);
</script>

<template>
  <div class="sl-server-selector" :class="{ collapsed }">
    <button
      ref="triggerRef"
      class="server-selector-trigger"
      :class="{ open: isOpen }"
      @click="toggleDropdown"
      @keydown="handleKeydown"
      :aria-label="placeholder"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
    >
      <Server :size="20" :stroke-width="1.8" class="server-icon" />
      <template v-if="!collapsed">
        <span class="server-label">{{ displayLabel }}</span>
        <ChevronDown :size="14" class="chevron" :class="{ open: isOpen }" />
      </template>
    </button>

    <Teleport to="body">
      <Transition name="dropdown">
        <div
          v-if="isOpen"
          ref="dropdownRef"
          class="server-selector-dropdown"
          :style="dropdownStyle"
          role="listbox"
        >
          <div class="server-selector-options">
            <div
              v-for="(option, index) in options"
              :key="option.value"
              class="server-selector-option"
              :class="{
                selected: option.value === modelValue,
                highlighted: index === highlightedIndex,
              }"
              @click="selectOption(option)"
              @mouseenter="highlightedIndex = index"
              role="option"
              :aria-selected="option.value === modelValue"
            >
              <span class="option-label">{{ option.label }}</span>
              <Check
                v-if="option.value === modelValue"
                class="check-icon"
                :size="16"
              />
            </div>
            <div v-if="options.length === 0" class="server-selector-empty">
              {{ i18n.t("common.no_servers") }}
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.sl-server-selector {
  width: 100%;
}

.server-selector-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: 8px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  min-height: 40px;
  margin-top: 5px;
}

.server-selector-trigger:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-color: var(--sl-primary-alpha);
}

.server-selector-trigger:hover .server-icon {
  color: var(--sl-primary);
}

.server-selector-trigger.open {
  background: var(--sl-primary-bg);
  border-color: var(--sl-primary-alpha);
}

.server-icon {
  flex-shrink: 0;
  color: var(--sl-text-secondary);
  transition: color var(--sl-transition-fast);
}

.server-label {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.875rem;
}

.chevron {
  flex-shrink: 0;
  color: var(--sl-text-tertiary);
  transition: transform var(--sl-transition-fast);
}

.chevron.open {
  transform: rotate(180deg);
}

.sl-server-selector.collapsed .server-selector-trigger {
  width: 40px;
  height: 40px;
  justify-content: center;
  padding: 0;
  border: none;
  background: transparent;
  margin-top: 0;
  margin-left: auto;
  margin-right: auto;
}

.sl-server-selector.collapsed .server-selector-trigger:hover {
  background: var(--sl-primary-bg);
}
</style>

<style>
.server-selector-dropdown {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  box-shadow: var(--sl-shadow-lg);
  overflow: hidden;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

[data-theme="dark"] .server-selector-dropdown {
  box-shadow:
    0 10px 30px rgba(0, 0, 0, 0.3),
    0 4px 12px rgba(0, 0, 0, 0.2);
}

.server-selector-options {
  max-height: 280px;
  overflow-y: auto;
  padding: var(--sl-space-xs);
}

.server-selector-options::-webkit-scrollbar {
  width: 6px;
}

.server-selector-options::-webkit-scrollbar-track {
  background: transparent;
}

.server-selector-options::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: var(--sl-radius-sm);
}

.server-selector-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  user-select: none;
}

.server-selector-option:hover,
.server-selector-option.highlighted {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.server-selector-option.selected {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-weight: 500;
}

.server-selector-option .option-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-selector-option .check-icon {
  flex-shrink: 0;
  color: var(--sl-primary);
  margin-left: 8px;
  animation: check-in 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes check-in {
  from {
    opacity: 0;
    transform: scale(0.5) rotate(-45deg);
  }
  to {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
}

.server-selector-empty {
  padding: 16px;
  text-align: center;
  color: var(--sl-text-tertiary);
  font-size: 0.875rem;
}

.dropdown-enter-active {
  animation: dropdown-enter 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dropdown-leave-active {
  animation: dropdown-leave 0.15s ease;
}

@keyframes dropdown-enter {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes dropdown-leave {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
}

.server-selector-option {
  animation: option-fade-in 0.2s ease backwards;
}

.server-selector-option:nth-child(1) { animation-delay: 0.02s; }
.server-selector-option:nth-child(2) { animation-delay: 0.04s; }
.server-selector-option:nth-child(3) { animation-delay: 0.06s; }
.server-selector-option:nth-child(4) { animation-delay: 0.08s; }
.server-selector-option:nth-child(5) { animation-delay: 0.1s; }
.server-selector-option:nth-child(6) { animation-delay: 0.12s; }
.server-selector-option:nth-child(7) { animation-delay: 0.14s; }
.server-selector-option:nth-child(8) { animation-delay: 0.16s; }

@keyframes option-fade-in {
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
