<script setup lang="ts">
interface Props {
  modelValue?: string;
  placeholder?: string;
  label?: string;
  type?: string;
  disabled?: boolean;
  maxlength?: number;
}

withDefaults(defineProps<Props>(), {
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
</script>

<template>
  <div class="sl-input-wrapper">
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
  color: #666;
}

.sl-input-container {
  display: flex;
  align-items: center;
  background: var(--sl-surface, #fff);
  border: 1px solid var(--sl-border, #ddd);
  border-radius: var(--sl-radius-sm);
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  overflow: hidden;
}

.sl-input-container:focus-within {
  border-color: #007aff;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

.sl-input {
  flex: 1;
  padding: 8px 12px;
  font-size: 14px;
  background: transparent;
  border: 0;
  outline: 0;
  min-width: 0;
}

.sl-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.sl-input::placeholder {
  color: #999;
}

.sl-input-prefix,
.sl-input-suffix {
  display: flex;
  align-items: center;
  padding: 0 8px;
  color: #999;
}
</style>
