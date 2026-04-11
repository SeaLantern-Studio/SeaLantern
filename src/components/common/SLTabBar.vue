<script setup lang="ts">
import { computed, watch, type Component } from "vue";
import { useTabIndicator } from "@composables/useTabIndicator";
import { i18n } from "@language";
import SLTooltip from "@components/common/SLTooltip.vue";

export interface TabBarItem<T = string | null> {
  key: T;
  label: string;
  count?: number | string;
  countTitle?: string;
  icon?: string;
  suffixIcon?: Component;
  suffixTitle?: string;
  disabled?: boolean;
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

function selectTab(tab: TabBarItem<string | null>) {
  if (!tab.disabled) {
    activeTab.value = tab.key;
  }
}
</script>

<template>
  <div class="sl-tab-bar" :class="`sl-tab-bar--level-${level}`">
    <!-- Level 1: 下划线风格 -->
    <div v-if="level === 1" class="sl-tab-bar__nav" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab.key ?? 'all'"
        type="button"
        class="sl-tab-bar__tab"
        :class="{
          'sl-tab-bar__tab--active': modelValue === tab.key,
          'sl-tab-bar__tab--disabled': tab.disabled,
        }"
        :disabled="tab.disabled"
        @click="selectTab(tab)"
        role="tab"
        :aria-selected="modelValue === tab.key"
        :aria-disabled="tab.disabled"
        :tabindex="modelValue === tab.key ? 0 : -1"
      >
        <i v-if="tab.icon" :class="tab.icon" class="sl-tab-bar__icon" aria-hidden="true" />
        <span class="sl-tab-bar__label">{{ tab.label }}</span>
        <span v-if="tab.suffixIcon" class="sl-tab-bar__suffix" :title="tab.suffixTitle">
          <component :is="tab.suffixIcon" class="sl-tab-bar__suffix-icon" :size="12" />
        </span>
        <SLTooltip v-if="tab.countTitle" :content="tab.countTitle">
          <span class="sl-tab-bar__count sl-tab-bar__count--hint" @click.stop @mousedown.stop>
            {{ tab.count }}
          </span>
        </SLTooltip>
        <span v-else-if="tab.count !== undefined" class="sl-tab-bar__count">{{ tab.count }}</span>
      </button>
      <div v-if="$slots.extra" class="sl-tab-bar__extra">
        <slot name="extra"></slot>
      </div>
    </div>

    <!-- Level 2: 药丸风格 -->
    <template v-else>
      <div class="sl-tab-bar__tabs">
        <div class="sl-tab-bar__indicator" ref="indicatorRef"></div>
        <button
          v-for="tab in tabs"
          :key="tab.key ?? 'all'"
          type="button"
          class="sl-tab-bar__btn"
          :class="{ active: modelValue === tab.key }"
          @click="selectTab(tab)"
        >
          <span class="sl-tab-bar__label">{{ tab.label }}</span>
          <span v-if="tab.suffixIcon" class="sl-tab-bar__suffix" :title="tab.suffixTitle">
            <component :is="tab.suffixIcon" class="sl-tab-bar__suffix-icon" :size="12" />
          </span>
          <SLTooltip v-if="tab.countTitle" :content="tab.countTitle">
            <span class="sl-tab-bar__count sl-tab-bar__count--hint" @click.stop @mousedown.stop>
              {{ tab.count }}
            </span>
          </SLTooltip>
          <span v-else-if="tab.count !== undefined" class="sl-tab-bar__count">{{ tab.count }}</span>
        </button>
      </div>
      <div v-if="$slots.extra" class="sl-tab-bar__extra">
        <slot name="extra"></slot>
      </div>
    </template>
  </div>
</template>

<style scoped>
.sl-tab-bar {
  display: flex;
  align-items: center;
  margin-bottom: var(--sl-space-md);
}

/* ===== Level 1: 下划线风格 ===== */
.sl-tab-bar__nav {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid var(--sl-border-light);
  overflow-x: auto;
  scrollbar-width: none;
  flex: 1;
}

.sl-tab-bar__nav::-webkit-scrollbar {
  display: none;
}

.sl-tab-bar__tab {
  --tab-underline-scale: 0;
  --tab-underline-opacity: 0;

  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 10px 16px;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-tertiary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  white-space: nowrap;
  margin-bottom: -1px;
  user-select: none;
  position: relative;
  transition: color 0.2s ease;
}

.sl-tab-bar__tab::after {
  content: "";
  position: absolute;
  bottom: -1px;
  left: 50%;
  width: 100%;
  height: 2px;
  background: var(--sl-primary);
  transform: translateX(-50%) scaleX(var(--tab-underline-scale));
  opacity: var(--tab-underline-opacity);
  transition:
    transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.25s ease;
}

.sl-tab-bar__tab:hover:not(:disabled):not(.sl-tab-bar__tab--active) {
  color: var(--sl-text-primary);
  --tab-underline-scale: 0.5;
  --tab-underline-opacity: 0.4;
}

.sl-tab-bar__tab--active {
  color: var(--sl-primary);
  --tab-underline-scale: 1;
  --tab-underline-opacity: 1;
}

.sl-tab-bar__tab--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sl-tab-bar__tab:active:not(:disabled) {
  transform: scale(0.98);
}

.sl-tab-bar__icon {
  font-size: 1rem;
  transition: transform 0.2s ease;
}

.sl-tab-bar__suffix {
  display: inline-flex;
  align-items: center;
  color: var(--sl-text-secondary);
}

.sl-tab-bar__suffix-icon {
  opacity: 0.85;
}

.sl-tab-bar__count--hint {
  cursor: help;
}

.sl-tab-bar__tab:hover:not(:disabled) .sl-tab-bar__icon {
  transform: scale(1.1);
}

.sl-tab-bar--level-1 .sl-tab-bar__count {
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
  transition:
    background 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;
}

.sl-tab-bar--level-1 .sl-tab-bar__tab--active .sl-tab-bar__count {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  transform: scale(1.05);
}

.sl-tab-bar--level-1 .sl-tab-bar__extra {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  margin-left: auto;
  flex-shrink: 0;
}

/* ===== Level 2: 药丸风格 ===== */
.sl-tab-bar--level-2 {
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
}

.sl-tab-bar__tabs {
  display: flex;
  gap: var(--sl-space-xs);
  background: transparent;
  border: none;
  padding: 0;
  position: relative;
  overflow: visible;
}

.sl-tab-bar__indicator {
  position: absolute;
  top: 0;
  bottom: 0;
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
  transition:
    left 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.15s ease;
}

.sl-tab-bar__btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 6px 12px;
  font-size: 0.8125rem;
  border-radius: var(--sl-radius-sm);
  font-weight: 500;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  position: relative;
  z-index: 2;
  white-space: nowrap;
  transition:
    color 0.2s ease,
    transform 0.15s ease;
}

.sl-tab-bar__btn:hover {
  color: var(--sl-text-primary);
}

.sl-tab-bar__btn:active {
  transform: scale(0.96);
}

.sl-tab-bar__btn.active {
  color: var(--sl-primary);
}

.sl-tab-bar--level-2 .sl-tab-bar__count {
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
  transition:
    background 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;
}

.sl-tab-bar__btn.active .sl-tab-bar__count {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  transform: scale(1.05);
}

.sl-tab-bar--level-2 .sl-tab-bar__extra {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  margin-left: auto;
  flex-shrink: 0;
}
</style>
