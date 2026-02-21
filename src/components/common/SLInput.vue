<script setup lang="ts">
import { ref } from "vue";
import { useRegisterComponent } from "../../composables/useRegisterComponent";

interface Props {
  modelValue?: string;
  placeholder?: string;
  label?: string;
  type?: string;
  disabled?: boolean;
  maxlength?: number;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: "",
  placeholder: "",
  type: "text",
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

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
</script>

<template>
  <div ref="elRef" class="sl-input-wrapper">
    <label v-if="label" class="sl-input-label">{{ label }}</label>
    <div class="sl-input-container">
      <div v-if="$slots.prefix" class="sl-input-prefix">
        <slot name="prefix" />
      </div>
      <input
        class="sl-input"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :maxlength="maxlength"
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
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sl-input-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--sl-text-secondary);
}

.sl-input-container {
  display: flex;
  align-items: center;
  background: var(--sl-surface, #fff);
  border: 1px solid var(--sl-border, #ddd);
  border-radius: 6px;
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
  padding: 8px 12px;
  font-size: 14px;
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
.sl-input[type="number"] {
  -moz-appearance: textfield;
}

.sl-input[type="number"]::-webkit-outer-spin-button,
.sl-input[type="number"]::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.sl-input-prefix,
.sl-input-suffix {
  display: flex;
  align-items: center;
  padding: 0 8px;
  color: var(--sl-text-tertiary);
}
</style>
