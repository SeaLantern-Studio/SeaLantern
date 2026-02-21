<script setup lang="ts">
import { ref } from "vue";
import { useRegisterComponent } from "../../composables/useRegisterComponent";

export interface TabItem {
  key: string;
  label: string;
  icon?: string;
  disabled?: boolean;
}

interface Props {
  modelValue?: string;
  tabs: TabItem[];
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: "",
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const handleTabClick = (tab: TabItem) => {
  if (!tab.disabled) {
    emit("update:modelValue", tab.key);
  }
};

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-tabs-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLTabs",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", String(value));
  },
  call: () => undefined,
  on: () => () => {},
  el: () => elRef.value,
});
</script>

<template>
  <div ref="elRef" class="sl-tabs">
    <div class="sl-tabs__nav" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="sl-tabs__tab"
        :class="{
          'sl-tabs__tab--active': modelValue === tab.key,
          'sl-tabs__tab--disabled': tab.disabled,
        }"
        :disabled="tab.disabled"
        @click="handleTabClick(tab)"
        role="tab"
        :aria-selected="modelValue === tab.key"
        :aria-disabled="tab.disabled"
        :tabindex="modelValue === tab.key ? 0 : -1"
      >
        <i v-if="tab.icon" :class="tab.icon" class="sl-tabs__icon" aria-hidden="true" />
        <span>{{ tab.label }}</span>
      </button>
    </div>
    <div class="sl-tabs__content" role="tabpanel">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.sl-tabs {
  display: flex;
  flex-direction: column;
}

.sl-tabs__nav {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid var(--sl-border, #e2e8f0);
  overflow-x: auto;
  scrollbar-width: none;
}

.sl-tabs__nav::-webkit-scrollbar {
  display: none;
}

.sl-tabs__tab {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs, 4px);
  padding: 10px 16px;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-tertiary, #94a3b8);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  white-space: nowrap;
  transition:
    color var(--sl-transition-fast, 0.15s ease),
    border-color var(--sl-transition-fast, 0.15s ease);
  margin-bottom: -1px;
  user-select: none;
}

.sl-tabs__tab:hover:not(:disabled) {
  color: var(--sl-text-primary, #0f172a);
}

.sl-tabs__tab--active {
  color: var(--sl-primary, #0ea5e9);
  border-bottom-color: var(--sl-primary, #0ea5e9);
}

.sl-tabs__tab--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sl-tabs__icon {
  font-size: 1rem;
}

.sl-tabs__content {
  padding-top: var(--sl-space-md, 16px);
}
</style>
