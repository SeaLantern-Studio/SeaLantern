<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Check, ChevronDown, Loader2, Search } from "lucide-vue-next";
import { i18n } from "@language";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Option {
  label: string;
  value: string | number;
  subLabel?: string;
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
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: () => i18n.t("common.select"),
  disabled: false,
  searchable: false,
  loading: false,
  maxHeight: "280px",
  previewFont: false,
});

const _selectId = props.componentId ?? `sl-select-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(_selectId, {
  type: "SLSelect",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", value as string | number);
  },
  call: () => undefined,
  on: () => () => {},
  el: () => containerRef.value,
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

const optionsMaxHeight = computed(() => {
    let maxHeight = parseInt(props.maxHeight) || 280;
    if (props.searchable) {
        maxHeight = Math.max(50, maxHeight - 50);
    }
    return `${maxHeight}px`;
});

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
  const viewportHeight = window.innerHeight;
  const dropdownMaxHeight = parseInt(props.maxHeight) || 280;
  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  const gap = 4;

  const openUpward = spaceBelow < dropdownMaxHeight + gap && spaceAbove > spaceBelow;

  if (openUpward) {
    dropdownStyle.value = {
      position: "fixed",
      bottom: `${viewportHeight - rect.top + gap}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      zIndex: "99999",
      maxHeight: `${Math.min(spaceAbove - gap, dropdownMaxHeight)}px`,
    };
  } else {
    dropdownStyle.value = {
      position: "fixed",
      top: `${rect.bottom + gap}px`,
      left: `${rect.left}px`,
      width: `${rect.width}px`,
      zIndex: "99999",
      maxHeight: `${Math.min(spaceBelow - gap, dropdownMaxHeight)}px`,
    };
  }
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
    if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown" || e.key === "ArrowUp") {
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

  const handleHome = () => {
    e.preventDefault();
    highlightedIndex.value = 0;
    scrollToHighlighted();
  };

  const handleEnd = () => {
    e.preventDefault();
    highlightedIndex.value = filteredOptions.value.length - 1;
    scrollToHighlighted();
  };

  const handleEnter = () => {
    e.preventDefault();
    if (highlightedIndex.value >= 0 && filteredOptions.value[highlightedIndex.value]) {
      selectOption(filteredOptions.value[highlightedIndex.value]);
    }
  };

  const handleSpace = () => {
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
    case "Home":
      handleHome();
      break;
    case "End":
      handleEnd();
      break;
    case "Enter":
      handleEnter();
      break;
    case " ":
      handleSpace();
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

const handleScroll = () => {
  if (isOpen.value) {
    updateDropdownPosition();
  }
};

const stopWatch = watch(searchQuery, () => {
  highlightedIndex.value = filteredOptions.value.length > 0 ? 0 : -1;
});

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("scroll", handleScroll, true);
  window.addEventListener("resize", handleScroll);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  window.removeEventListener("scroll", handleScroll, true);
  window.removeEventListener("resize", handleScroll);
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
      :aria-owns="isOpen ? 'sl-select-listbox' : undefined"
      :aria-activedescendant="
        isOpen && highlightedIndex >= 0
          ? `option-${filteredOptions[highlightedIndex].value}`
          : undefined
      "
    >
      <span v-if="loading" class="sl-select-loading" aria-live="polite">
        <Loader2 class="spinner" :size="16" aria-hidden="true" />
        {{ i18n.t("common.loading") }}
      </span>
      <span
        v-else-if="selectedOption"
        class="sl-select-value"
        :style="getFontStyle(selectedOption.value)"
      >
        {{ selectedOption.label }}
      </span>
      <span v-else class="sl-select-placeholder">{{ placeholder }}</span>

      <ChevronDown
        class="sl-select-arrow"
        :class="{ open: isOpen }"
        :size="16"
        aria-hidden="true"
      />
    </div>

    <Teleport to="body">
      <Transition name="dropdown">
        <div v-if="isOpen" class="sl-select-dropdown" ref="dropdownRef" :style="dropdownStyle">
          <div v-if="searchable" class="sl-select-search">
            <Search class="search-icon" :size="16" aria-hidden="true" />
            <input
              ref="inputRef"
              v-model="searchQuery"
              type="text"
              :placeholder="i18n.t('common.search')"
              class="sl-select-input"
              @keydown="handleKeydown"
              :aria-label="i18n.t('common.search_options')"
            />
          </div>

          <div
            id="sl-select-listbox"
            class="sl-select-options"
            :style="{ maxHeight: optionsMaxHeight }"
            role="listbox"
            :aria-activedescendant="
              highlightedIndex >= 0
                ? `option-${filteredOptions[highlightedIndex].value}`
                : undefined
            "
          >
            <div v-if="filteredOptions.length === 0" class="sl-select-empty" role="presentation">
              {{ i18n.t("common.no_match") }}
            </div>
            <div
              v-for="(option, index) in filteredOptions"
              :key="option.value"
              :id="`option-${option.value}`"
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
              tabindex="-1"
            >
              <span class="option-label-wrap">
                <span class="option-label">{{ option.label }}</span>
                <span v-if="option.subLabel" class="option-sublabel">{{ option.subLabel }}</span>
              </span>
              <Check
                v-if="option.value === modelValue"
                class="check-icon"
                :size="16"
                aria-hidden="true"
              />
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
  background: var(--sl-surface, #1e2130);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  box-shadow: var(--sl-shadow-lg);
  overflow: hidden;
  backdrop-filter: blur(20px);
  will-change: transform, opacity;
  color: var(--sl-text-primary);
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
  border-radius: 3px;
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

.sl-select-dropdown .sl-select-option .option-label-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.sl-select-dropdown .sl-select-option .option-sublabel {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  font-family: var(--sl-font-mono);
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
