<script setup lang="ts">
import { computed, watch } from "vue";
import { useTabIndicator } from "@composables/useTabIndicator";
import { i18n } from "@language";

export interface TabBarItem<T = string | null> {
  key: T;
  label: string;
  count?: number;
}

interface Props<T = string | null> {
  modelValue: T;
  tabs: TabBarItem<T>[];
  level?: 1 | 2;
}

const props = withDefaults(defineProps<Props<string | null>>(), {
  level: 1,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string | null): void;
}>();

const activeTab = computed({
  get: () => props.modelValue,
  set: (value: string | null) => emit("update:modelValue", value),
});

const { indicatorRef, updatePosition } = useTabIndicator(activeTab);

const localeRef = i18n.getLocaleRef();
watch(localeRef, () => {
  updatePosition();
});

function selectTab(key: string | null) {
  activeTab.value = key;
}
</script>

<template>
  <div class="sl-tab-bar" :class="`sl-tab-bar--level-${level}`">
    <div class="sl-tab-bar__tabs">
      <div class="sl-tab-bar__indicator" ref="indicatorRef"></div>
      <button
        v-for="tab in tabs"
        :key="tab.key ?? 'all'"
        type="button"
        class="sl-tab-bar__btn"
        :class="{ active: modelValue === tab.key }"
        @click="selectTab(tab.key)"
      >
        <span class="sl-tab-bar__label">{{ tab.label }}</span>
        <span v-if="tab.count !== undefined" class="sl-tab-bar__count">{{ tab.count }}</span>
      </button>
    </div>
    <div v-if="$slots.extra" class="sl-tab-bar__extra">
      <slot name="extra"></slot>
    </div>
  </div>
</template>

<style scoped>
.sl-tab-bar {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-md);
}

.sl-tab-bar__tabs {
  display: flex;
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
  position: relative;
  overflow: hidden;
}

.sl-tab-bar__indicator {
  position: absolute;
  top: var(--sl-space-xs);
  bottom: var(--sl-space-xs);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}

.sl-tab-bar__btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  border-radius: var(--sl-radius-sm);
  font-weight: 500;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  white-space: nowrap;
}

.sl-tab-bar__btn:hover {
  color: var(--sl-text-primary);
}

.sl-tab-bar__btn.active {
  color: var(--sl-primary);
}

.sl-tab-bar__count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.sl-tab-bar__btn.active .sl-tab-bar__count {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.sl-tab-bar__extra {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  flex-shrink: 0;
}

/* 一级Tab栏样式 */
.sl-tab-bar--level-1 .sl-tab-bar__btn {
  padding: 10px 16px;
  font-size: 0.875rem;
}

/* 二级Tab栏样式 - 整体白色背景 */
.sl-tab-bar--level-2 {
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
}

.sl-tab-bar--level-2 .sl-tab-bar__tabs {
  background: transparent;
  border: none;
  padding: 0;
  overflow: visible;
}

.sl-tab-bar--level-2 .sl-tab-bar__indicator {
  top: 0;
  bottom: 0;
}

.sl-tab-bar--level-2 .sl-tab-bar__btn {
  padding: 6px 12px;
  font-size: 0.8125rem;
}

.sl-tab-bar--level-2 .sl-tab-bar__extra {
  margin-left: auto;
}
</style>
