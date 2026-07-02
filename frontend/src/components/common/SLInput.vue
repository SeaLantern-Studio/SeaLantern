<script setup lang="ts">
import { computed, ref } from "vue";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Props {
  modelValue?: string;
  placeholder?: string;
  label?: string;
  type?: string;
  disabled?: boolean;
  maxlength?: number;
  min?: number | string;
  max?: number | string;
  step?: number | string;
  hideNumberControls?: boolean;
  size?: "sm" | "md" | "lg";
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: "",
  placeholder: "",
  type: "text",
  disabled: false,
  hideNumberControls: true,
  size: "md",
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const wrapperClasses = computed(() => [`sl-input-wrapper--${props.size}`]);

const handleInput = (e: Event) => {
  emit("update:modelValue", (e.target as HTMLInputElement).value);
};

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-input-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLInput",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", String(value ?? ""));
  },
  call: () => undefined,
  on: () => () => {},
  el: () => elRef.value,
});

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>

<template>
  <div ref="elRef" class="sl-input-wrapper" :class="wrapperClasses">
    <label v-if="label" class="sl-input-label">{{ label }}</label>
    <div class="sl-input-container">
      <div v-if="$slots.prefix" class="sl-input-prefix">
        <slot name="prefix" />
      </div>
      <input
        ref="inputRef"
        class="sl-input"
        :class="{ 'sl-input--hide-number-controls': hideNumberControls }"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :maxlength="maxlength"
        :min="min"
        :max="max"
        :step="step"
        @input="handleInput"
      />
      <div v-if="$slots.suffix" class="sl-input-suffix">
        <slot name="suffix" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.sl-input-wrapper {
  --sl-input-padding-y: 8px;
  --sl-input-padding-x: 12px;
  --sl-input-font-size: 14px;
  --sl-input-min-height: 38px;
  --sl-input-addon-padding-x: 8px;
  --sl-input-action-padding-y: 4px;
  --sl-input-action-padding-x: 10px;
  --sl-input-action-font-size: var(--sl-font-size-sm, 0.875rem);

  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sl-input-wrapper--sm {
  --sl-input-padding-y: 6px;
  --sl-input-padding-x: 10px;
  --sl-input-font-size: 13px;
  --sl-input-min-height: 34px;
  --sl-input-addon-padding-x: 8px;
  --sl-input-action-padding-y: 3px;
  --sl-input-action-padding-x: 8px;
  --sl-input-action-font-size: var(--sl-font-size-xs, 0.75rem);
}

.sl-input-wrapper--lg {
  --sl-input-padding-y: 10px;
  --sl-input-padding-x: 14px;
  --sl-input-font-size: 16px;
  --sl-input-min-height: 44px;
  --sl-input-addon-padding-x: 10px;
  --sl-input-action-padding-y: 5px;
  --sl-input-action-padding-x: 12px;
  --sl-input-action-font-size: var(--sl-font-size-base, 1rem);
}

.sl-input-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--sl-text-secondary);
}

.sl-input-container {
  display: flex;
  align-items: center;
  min-height: var(--sl-input-min-height);
  background: var(--sl-surface, #fff);
  border: 1px solid var(--sl-border, #ddd);
  border-radius: var(--sl-radius-sm);
  transition:
    border-color var(--sl-transition-fast),
    box-shadow var(--sl-transition-fast);
  overflow: hidden;
  will-change: border-color, box-shadow;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.sl-input-container:focus-within {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
}

.sl-input {
  flex: 1;
  padding: var(--sl-input-padding-y) var(--sl-input-padding-x);
  font-size: var(--sl-input-font-size);
  background: transparent;
  border: 0;
  outline: 0;
  min-width: 0;
  color: var(--sl-text-primary);
}

.sl-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.sl-input::placeholder {
  color: var(--sl-text-tertiary);
}

/* 禁用数字输入框的上下箭头 */
.sl-input--hide-number-controls[type="number"] {
  -moz-appearance: textfield;
}

.sl-input--hide-number-controls[type="number"]::-webkit-outer-spin-button,
.sl-input--hide-number-controls[type="number"]::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.sl-input-prefix,
.sl-input-suffix {
  display: flex;
  align-items: center;
  padding: 0 var(--sl-input-addon-padding-x);
  color: var(--sl-text-tertiary);
}

/* 统一的输入框内嵌操作按钮样式 */
:deep(.sl-input-action) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-input-action-padding-y) var(--sl-input-action-padding-x);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  font-size: var(--sl-input-action-font-size);
  font-family: inherit;
  line-height: 1.2;
  cursor: pointer;
  border: none;
  transition:
    background-color 0.15s ease,
    opacity 0.15s ease;
}

:deep(.sl-input-action:hover) {
  background: color-mix(in srgb, var(--sl-primary) 15%, var(--sl-primary-bg));
}

:deep(.sl-input-action:disabled) {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
