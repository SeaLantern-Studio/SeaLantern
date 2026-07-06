<script setup lang="ts">
interface Props {
  modelValue?: string;
  placeholder?: string;
  disabled?: boolean;
  rows?: number;
  maxlength?: number;
  resize?: "none" | "vertical" | "horizontal" | "both";
  size?: "sm" | "md" | "lg";
}

withDefaults(defineProps<Props>(), {
  modelValue: "",
  placeholder: "",
  disabled: false,
  rows: 3,
  resize: "vertical",
  size: "md",
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const handleInput = (e: Event) => {
  emit("update:modelValue", (e.target as HTMLTextAreaElement).value);
};
</script>

<template>
  <div class="sl-textarea-wrapper" :class="`sl-textarea-wrapper--${size}`">
    <textarea
      class="sl-textarea"
      :class="{ 'sl-textarea--disabled': disabled }"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows"
      :maxlength="maxlength"
      :style="{ resize }"
      @input="handleInput"
    />
    <span v-if="maxlength" class="sl-textarea-count">
      {{ (modelValue ?? "").length }} / {{ maxlength }}
    </span>
  </div>
</template>

<style scoped>
.sl-textarea-wrapper {
  --sl-textarea-padding-y: 8px;
  --sl-textarea-padding-x: 12px;
  --sl-textarea-font-size: 0.875rem;

  position: relative;
  display: flex;
  flex-direction: column;
}

.sl-textarea-wrapper--sm {
  --sl-textarea-padding-y: 6px;
  --sl-textarea-padding-x: 10px;
  --sl-textarea-font-size: 0.8125rem;
}

.sl-textarea-wrapper--lg {
  --sl-textarea-padding-y: 10px;
  --sl-textarea-padding-x: 14px;
  --sl-textarea-font-size: 1rem;
}

.sl-textarea {
  width: 100%;
  padding: var(--sl-textarea-padding-y) var(--sl-textarea-padding-x);
  font-size: var(--sl-textarea-font-size);
  font-family: inherit;
  line-height: 1.5;
  color: var(--sl-text-primary, #0f172a);
  background: var(--sl-surface, #ffffff);
  border: 1px solid var(--sl-border, #e2e8f0);
  border-radius: var(--sl-radius-md, 10px);
  outline: none;
  transition:
    border-color var(--sl-transition-fast, 0.15s ease),
    box-shadow var(--sl-transition-fast, 0.15s ease);
  box-sizing: border-box;
}

.sl-textarea:focus {
  border-color: var(--sl-primary, #0ea5e9);
  box-shadow: 0 0 0 3px var(--sl-primary-bg, rgba(14, 165, 233, 0.08));
}

.sl-textarea::placeholder {
  color: var(--sl-text-tertiary, #94a3b8);
}

.sl-textarea--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background-color: var(--sl-bg-tertiary, #f1f5f9);
  color: var(--sl-text-tertiary, #94a3b8);
  pointer-events: none;
  user-select: none;
  border-color: var(--sl-border, #e2e8f0);
}

.sl-textarea--disabled:focus {
  border-color: var(--sl-border, #e2e8f0);
  box-shadow: none;
}

.sl-textarea-count {
  align-self: flex-end;
  margin-top: 4px;
  font-size: 0.75rem;
  color: var(--sl-text-tertiary, #94a3b8);
}
</style>
