<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Listbox, ListboxButton, ListboxOptions, ListboxOption } from "@headlessui/vue";
import { Check, ChevronDown, Loader2, Search } from "lucide-vue-next";
import { i18n } from "../../locales";

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
  size?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: () => i18n.t("common.select"),
  disabled: false,
  searchable: false,
  loading: false,
  maxHeight: "280px",
  previewFont: false,
  size: undefined,
});

const emit = defineEmits(["update:modelValue"]);

const internalValue = computed({
  get: () => props.modelValue,
  set: (v: any) => emit("update:modelValue", v),
});

const searchQuery = ref("");

const filteredOptions = computed(() => {
  if (!props.searchable || !searchQuery.value.trim()) return props.options;
  const q = searchQuery.value.toLowerCase();
  return props.options.filter((o) => o.label.toLowerCase().includes(q) || (o.subLabel || "").toLowerCase().includes(q));
});

const selectedOption = computed(() => props.options.find((o) => o.value === props.modelValue));

const buttonRef = ref<HTMLElement | null>(null);
const containerRef = ref<HTMLElement | null>(null);
const optionsRef = ref<HTMLElement | null>(null);
const dropdownStyle = ref<Record<string, string>>({});

const sizeClass = computed(() => {
  if (!props.size) return "";
  return `sl-select--${String(props.size)}`;
});

const updateDropdownPosition = () => {
  if (!optionsRef.value) return;
  const el = (buttonRef.value && typeof (buttonRef.value as any).getBoundingClientRect === "function")
    ? buttonRef.value
    : containerRef.value;
  if (!el || typeof (el as any).getBoundingClientRect !== "function") return;
  const rect = (el as HTMLElement).getBoundingClientRect();
  const viewportHeight = window.innerHeight;
  const dropdownMaxHeight = parseInt(String(props.maxHeight)) || 280;
  const spaceBelow = viewportHeight - rect.bottom;
  const spaceAbove = rect.top;
  const gap = 6;

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
  try {
    if (optionsRef.value) optionsRef.value.style.maxHeight = dropdownStyle.value.maxHeight || props.maxHeight || "280px";
  } catch (e) {
    // ignore
  }
};

let highlightObserver: MutationObserver | null = null;
let keydownListener: ((e: KeyboardEvent) => void) | null = null;

const resolveDom = (node: any): HTMLElement | null => {
  if (!node) return null;
  if (node instanceof HTMLElement) return node;
  if (node.$el && node.$el instanceof HTMLElement) return node.$el as HTMLElement;
  return null;
};

const scrollToHighlighted = () => {
  const opts = resolveDom(optionsRef.value) as HTMLElement | null;
  if (!opts) return;
  const highlighted = opts.querySelector('.sl-select-option.highlighted') as HTMLElement | null;
  if (highlighted) highlighted.scrollIntoView({ block: 'nearest', behavior: 'auto' });
};

watch(optionsRef, (el) => {
  const optsEl = resolveDom(el);
  if (!optsEl) return;
  nextTick(() => updateDropdownPosition());
  // retry a couple times to handle async render/transition timing
  setTimeout(() => updateDropdownPosition(), 50);
  requestAnimationFrame(() => updateDropdownPosition());

  // disconnect previous observer/listener
  if (highlightObserver) {
    highlightObserver.disconnect();
    highlightObserver = null;
  }
  if (keydownListener) {
    document.removeEventListener('keydown', keydownListener);
    keydownListener = null;
  }

  // observe class changes to detect highlighted item
  highlightObserver = new MutationObserver(() => {
    scrollToHighlighted();
  });
  highlightObserver.observe(optsEl, { subtree: true, attributes: true, attributeFilter: ['class'] });

  // ensure arrow key navigation scrolls
  keydownListener = (e: KeyboardEvent) => {
    const keys = ['ArrowDown', 'ArrowUp', 'Home', 'End', 'PageDown', 'PageUp'];
    if (keys.includes(e.key)) {
      // slightly defer to let Headless UI update active class
      setTimeout(() => scrollToHighlighted(), 0);
    }
  };
  document.addEventListener('keydown', keydownListener);
});

const onScrollOrResize = () => {
  if (optionsRef.value) updateDropdownPosition();
};

onMounted(() => {
  window.addEventListener("scroll", onScrollOrResize, true);
  window.addEventListener("resize", onScrollOrResize);
});

onUnmounted(() => {
  window.removeEventListener("scroll", onScrollOrResize, true);
  window.removeEventListener("resize", onScrollOrResize);
});
</script>

<template>
  <div class="sl-select" ref="containerRef" :class="sizeClass">
    <label v-if="props.label" class="sl-select-label">{{ props.label }}</label>

    <Listbox v-model="internalValue" :disabled="props.disabled" v-slot="{ open }">
      <div>
        <ListboxButton as="div" ref="buttonRef" class="sl-select-trigger" :class="{ disabled: props.disabled, open }" @mousedown.prevent>
          <span v-if="props.loading" class="sl-select-loading" aria-live="polite">
            <Loader2 class="spinner" :size="16" aria-hidden="true" />
            {{ i18n.t("common.loading") }}
          </span>
          <span v-else-if="selectedOption" class="sl-select-value" :style="props.previewFont ? { fontFamily: String(selectedOption.value) } : {}">
            {{ selectedOption.label }}
          </span>
          <span v-else class="sl-select-placeholder">{{ props.placeholder }}</span>

          <ChevronDown class="sl-select-arrow" :class="{ open }" :size="16" aria-hidden="true" />
        </ListboxButton>

        <Teleport to="body">
          <transition name="dropdown">
            <ListboxOptions ref="optionsRef" class="sl-select-dropdown sl-select-options" :style="dropdownStyle" tabindex="-1">
              <div v-if="props.searchable" class="sl-select-search">
                <Search class="search-icon" :size="16" />
                <input v-model="searchQuery" :placeholder="i18n.t('common.search')" class="sl-select-input" />
              </div>

              <div v-if="props.loading" class="sl-select-loading">
                <Loader2 class="sl-spin" :size="18" />
                <span>{{ i18n.t('common.loading') }}</span>
              </div>

              <template v-else>
                <ListboxOption
                  v-for="opt in filteredOptions"
                  :key="opt.value"
                  :value="opt.value"
                  v-slot="{ active, selected }"
                >
                  <div :class="['sl-select-option', { selected, highlighted: active }]" :style="props.previewFont ? { fontFamily: String(opt.value) } : {}">
                    <span class="option-label-wrap">
                      <span class="option-label">{{ opt.label }}</span>
                      <span v-if="opt.subLabel" class="option-sublabel">{{ opt.subLabel }}</span>
                    </span>
                    <Check v-if="selected" class="check-icon" :size="16" aria-hidden="true" />
                  </div>
                </ListboxOption>

                <div v-if="!filteredOptions.length && !props.loading" class="sl-select-empty">
                  {{ i18n.t('common.no_match') }}
                </div>
              </template>
            </ListboxOptions>
          </transition>
        </Teleport>
      </div>
    </Listbox>
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

/* 移除默认 focus 边框，避免第一次打开时出现红色轮廓 */
.sl-select-dropdown:focus,
.sl-select-dropdown:focus-visible {
  outline: none;
  box-shadow: none;
  border-color: var(--sl-border);
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

.sl-select-dropdown .sl-select-options,
.sl-select-options {
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
