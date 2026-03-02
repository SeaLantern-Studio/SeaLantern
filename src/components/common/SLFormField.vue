<script setup lang="ts">
interface Props {
  label?: string;
  required?: boolean;
  error?: string;
  hint?: string;
  labelPosition?: "top" | "left";
}

withDefaults(defineProps<Props>(), {
  required: false,
  labelPosition: "top",
});
</script>

<template>
  <div
    class="sl-form-field"
    :class="[`sl-form-field--${labelPosition}`, { 'sl-form-field--error': error }]"
  >
    <label v-if="label" class="sl-form-field__label">
      {{ label }}
      <span v-if="required" class="sl-form-field__required" aria-hidden="true">*</span>
    </label>
    <div class="sl-form-field__content">
      <slot />
      <p v-if="error" class="sl-form-field__error" role="alert">{{ error }}</p>
      <p v-else-if="hint" class="sl-form-field__hint">{{ hint }}</p>
    </div>
  </div>
</template>

<style scoped>
.sl-form-field {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs, 4px);
}

.sl-form-field--left {
  flex-direction: row;
  align-items: flex-start;
  gap: var(--sl-space-md, 16px);
}

.sl-form-field--left .sl-form-field__label {
  min-width: 100px;
  padding-top: 8px;
  text-align: right;
  flex-shrink: 0;
}

.sl-form-field__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary, #475569);
  line-height: 1.4;
  user-select: none;
}

.sl-form-field__required {
  color: var(--sl-error, #ef4444);
  margin-left: 2px;
}

.sl-form-field__content {
  flex: 1;
  min-width: 0;
}

.sl-form-field__error {
  margin: 4px 0 0;
  font-size: 0.75rem;
  color: var(--sl-error, #ef4444);
  line-height: 1.4;
}

.sl-form-field__hint {
  margin: 4px 0 0;
  font-size: 0.75rem;
  color: var(--sl-text-tertiary, #94a3b8);
  line-height: 1.4;
}

.sl-form-field--error :deep(input),
.sl-form-field--error :deep(textarea),
.sl-form-field--error :deep(.sl-input-container) {
  border-color: var(--sl-error, #ef4444);
}

.sl-form-field--error :deep(input:focus),
.sl-form-field--error :deep(textarea:focus),
.sl-form-field--error :deep(.sl-input-container:focus-within) {
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}
</style>
