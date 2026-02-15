<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";

interface Option {
  label: string;
  value: string | number;
}

interface Props {
  modelValue?: string | number;
  options: Option[];
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  searchable?: boolean;
  loading?: boolean;
  maxHeight?: string;
  previewFont?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: "请选择",
  disabled: false,
  searchable: false,
  loading: false,
  maxHeight: "280px",
  previewFont: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
}>();

const isOpen = ref(false);
const searchQuery = ref("");
const containerRef = ref<HTMLElement | null>(null);
const dropdownRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLInputElement | null>(null);
const highlightedIndex = ref(-1);

const dropdownStyle = ref<Record<string, string>>({});

const getFontStyle = (value: string | number) => {
  if (!props.previewFont || !value) return {};
  return { fontFamily: String(value) };
};

const selectedOption = computed(() => {
  const currentValue = props.modelValue;
  if (currentValue === undefined) return undefined;
  return props.options.find((opt: Option) => opt.value === currentValue);
});

const filteredOptions = computed(() => {
  if (!props.searchable || !searchQuery.value.trim()) return props.options;

  const query = searchQuery.value.toLowerCase();
  return props.options.filter((opt: Option) => opt.label.toLowerCase().includes(query));
});

const updateDropdownPosition = () => {
  if (!containerRef.value) return;
  const rect = containerRef.value.getBoundingClientRect();
  dropdownStyle.value = {
    position: "fixed",
    top: `${rect.bottom + 4}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    zIndex: "99999",
  };
};

const toggleDropdown = () => {
  if (props.disabled) return;

  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    searchQuery.value = "";
    highlightedIndex.value = -1;
    nextTick(() => {
      updateDropdownPosition();
      if (props.searchable) {
        inputRef.value?.focus();
      }
    });
  }
};

const selectOption = (option: Option) => {
  emit("update:modelValue", option.value);
  isOpen.value = false;
  searchQuery.value = "";
};

const handleKeydown = (e: KeyboardEvent) => {
  if (!isOpen.value) {
    if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
      e.preventDefault();
      toggleDropdown();
    }
    return;
  }

  const handleArrowDown = () => {
    e.preventDefault();
    highlightedIndex.value = Math.min(highlightedIndex.value + 1, filteredOptions.value.length - 1);
    scrollToHighlighted();
  };

  const handleArrowUp = () => {
    e.preventDefault();
    highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0);
    scrollToHighlighted();
  };

  const handleEnter = () => {
    e.preventDefault();
    if (highlightedIndex.value >= 0 && filteredOptions.value[highlightedIndex.value]) {
      selectOption(filteredOptions.value[highlightedIndex.value]);
    }
  };

  switch (e.key) {
    case "ArrowDown":
      handleArrowDown();
      break;
    case "ArrowUp":
      handleArrowUp();
      break;
    case "Enter":
      handleEnter();
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
    containerRef.value &&
    !containerRef.value.contains(target) &&
    dropdownRef.value &&
    !dropdownRef.value.contains(target)
  ) {
    isOpen.value = false;
  }
};

const stopWatch = watch(searchQuery, () => {
  highlightedIndex.value = filteredOptions.value.length > 0 ? 0 : -1;
});

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  stopWatch();

  containerRef.value = null;
  dropdownRef.value = null;
  inputRef.value = null;
});
</script>

<template>
  <div class="sl-select" ref="containerRef">
    <label v-if="label" class="sl-select-label">{{ label }}</label>

    <div
      class="sl-select-trigger"
      :class="{ open: isOpen, disabled }"
      @click="toggleDropdown"
      @keydown="handleKeydown"
      tabindex="0"
      role="combobox"
      :aria-expanded="isOpen"
      :aria-disabled="disabled"
    >
      <span v-if="loading" class="sl-select-loading" aria-live="polite">
        <svg class="spinner" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" />
        </svg>
        加载中...
      </span>
      <span
        v-else-if="selectedOption"
        class="sl-select-value"
        :style="getFontStyle(selectedOption.value)"
      >
        {{ selectedOption.label }}
      </span>
      <span v-else class="sl-select-placeholder">{{ placeholder }}</span>

      <svg
        class="sl-select-arrow"
        :class="{ open: isOpen }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </div>

    <Teleport to="body">
      <Transition name="dropdown">
        <div v-if="isOpen" class="sl-select-dropdown" ref="dropdownRef" :style="dropdownStyle">
          <div v-if="searchable" class="sl-select-search">
            <svg
              class="search-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              aria-hidden="true"
            >
              <circle cx="11" cy="11" r="8" />
              <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
              ref="inputRef"
              v-model="searchQuery"
              type="text"
              placeholder="搜索..."
              class="sl-select-input"
              @keydown="handleKeydown"
              aria-label="搜索选项"
            />
          </div>

          <div class="sl-select-options" :style="{ maxHeight }" role="presentation">
            <div v-if="filteredOptions.length === 0" class="sl-select-empty">未找到匹配项</div>
            <div
              v-for="(option, index) in filteredOptions"
              :key="option.value"
              class="sl-select-option"
              :class="{
                selected: option.value === modelValue,
                highlighted: index === highlightedIndex,
              }"
              :style="getFontStyle(option.value)"
              @click="selectOption(option)"
              @mouseenter="highlightedIndex = index"
              role="option"
              :aria-selected="option.value === modelValue"
            >
              <span class="option-label">{{ option.label }}</span>
              <svg
                v-if="option.value === modelValue"
                class="check-icon"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                aria-hidden="true"
              >
                <path d="M20 6L9 17l-5-5" />
              </svg>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.sl-select {
  position: relative;
  width: 100%;
}

.sl-select-label {
  display: block;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-xs);
}

.sl-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 8px 12px;
  font-size: 0.875rem;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  color: var(--sl-text-primary);
  min-height: 38px;
  box-sizing: border-box;
}

.sl-select-trigger:hover:not(.disabled) {
  border-color: var(--sl-border-hover);
}

.sl-select-trigger:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
  outline: none;
}

.sl-select-trigger.open {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
}

.sl-select-trigger.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sl-select-value {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sl-select-placeholder {
  color: var(--sl-text-tertiary);
  flex: 1;
}

.sl-select-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--sl-text-tertiary);
  flex: 1;
}

.sl-select-loading .spinner {
  animation: spin 1s linear infinite;
  transform-origin: center;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.sl-select-arrow {
  color: var(--sl-text-tertiary);
  transition: transform var(--sl-transition-fast);
  flex-shrink: 0;
  margin-left: 8px;
}

.sl-select-arrow.open {
  transform: rotate(180deg);
}
</style>

<style>
/* 下拉框样式 - 非 scoped，因为使用 Teleport 渲染到 body */
.sl-select-dropdown {
  background: #1e2130;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  backdrop-filter: blur(20px);
  will-change: transform, opacity;
}

:root[data-theme="light"] .sl-select-dropdown {
  background: var(--sl-surface, #ffffff);
}

:root[data-acrylic="true"][data-theme="dark"] .sl-select-dropdown {
  background: rgba(30, 33, 48, 0.95);
}

:root[data-acrylic="true"][data-theme="light"] .sl-select-dropdown {
  background: rgba(255, 255, 255, 0.95);
}

.sl-select-dropdown .sl-select-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--sl-border);
}

.sl-select-dropdown .search-icon {
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

.sl-select-dropdown .sl-select-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 0.875rem;
  color: var(--sl-text-primary);
  outline: none;
  width: 100%;
}

.sl-select-dropdown .sl-select-input::placeholder {
  color: var(--sl-text-tertiary);
}

.sl-select-dropdown .sl-select-options {
  overflow-y: auto;
  overscroll-behavior: contain;
  -webkit-overflow-scrolling: touch;
}

.sl-select-dropdown .sl-select-options::-webkit-scrollbar {
  width: 6px;
}

.sl-select-dropdown .sl-select-options::-webkit-scrollbar-track {
  background: transparent;
}

.sl-select-dropdown .sl-select-options::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: var(--sl-radius-sm);
}

.sl-select-dropdown .sl-select-options::-webkit-scrollbar-thumb:hover {
  background: var(--sl-text-tertiary);
}

.sl-select-dropdown .sl-select-empty {
  padding: 16px;
  text-align: center;
  color: var(--sl-text-tertiary);
  font-size: 0.875rem;
}

.sl-select-dropdown .sl-select-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  cursor: pointer;
  transition: background var(--sl-transition-fast);
  user-select: none;
}

.sl-select-dropdown .sl-select-option:hover,
.sl-select-dropdown .sl-select-option.highlighted {
  background: var(--sl-surface-hover);
}

.sl-select-dropdown .sl-select-option.selected {
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.sl-select-dropdown .sl-select-option .option-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sl-select-dropdown .sl-select-option .check-icon {
  color: var(--sl-primary);
  flex-shrink: 0;
  margin-left: 8px;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease;
  will-change: transform, opacity;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

@media (max-width: 768px) {
  .sl-select-trigger {
    min-height: 44px;
    font-size: 16px;
  }

  .sl-select-option {
    min-height: 44px;
  }
}
</style>
