<script setup lang="ts">
import { ref } from "vue";
import { Check, Minus } from "lucide-vue-next";
import { useRegisterComponent } from "../../composables/useRegisterComponent";

interface Props {
  modelValue?: boolean;
  disabled?: boolean;
  label?: string;
  indeterminate?: boolean;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  disabled: false,
  indeterminate: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const handleChange = () => {
  if (!props.disabled) {
    emit("update:modelValue", !props.modelValue);
  }
};

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-checkbox-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLCheckbox",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", !!value);
  },
  call: () => undefined,
  on: () => () => {},
  el: () => elRef.value,
});
</script>

<template>
  <label ref="elRef" class="sl-checkbox" :class="{ 'sl-checkbox--disabled': disabled }">
    <div
      class="sl-checkbox__box"
      :class="{
        'sl-checkbox__box--checked': modelValue,
        'sl-checkbox__box--indeterminate': indeterminate && !modelValue,
      }"
      @click="handleChange"
      @keydown.space.prevent="handleChange"
      role="checkbox"
      :aria-checked="indeterminate ? 'mixed' : modelValue"
      :aria-disabled="disabled"
      tabindex="0"
    >
      <Check
        v-if="modelValue"
        class="sl-checkbox__icon"
        :size="12"
        :stroke-width="3"
        aria-hidden="true"
      />
      <Minus
        v-else-if="indeterminate"
        class="sl-checkbox__icon"
        :size="12"
        :stroke-width="3"
        aria-hidden="true"
      />
    </div>
    <span v-if="label" class="sl-checkbox__label">{{ label }}</span>
  </label>
</template>

<style scoped>
.sl-checkbox {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-sm, 8px);
  cursor: pointer;
  user-select: none;
}

.sl-checkbox--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sl-checkbox__box {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1.5px solid var(--sl-border, #e2e8f0);
  border-radius: var(--sl-radius-sm, 6px);
  background: var(--sl-surface, #ffffff);
  transition:
    background-color var(--sl-transition-fast, 0.15s ease),
    border-color var(--sl-transition-fast, 0.15s ease),
    box-shadow var(--sl-transition-fast, 0.15s ease);
  flex-shrink: 0;
}

.sl-checkbox__box:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px var(--sl-primary-bg, rgba(14, 165, 233, 0.08));
  border-color: var(--sl-primary, #0ea5e9);
}

.sl-checkbox__box:hover:not(.sl-checkbox--disabled .sl-checkbox__box) {
  border-color: var(--sl-primary, #0ea5e9);
}

.sl-checkbox__box--checked,
.sl-checkbox__box--indeterminate {
  background: var(--sl-primary, #0ea5e9);
  border-color: var(--sl-primary, #0ea5e9);
}

.sl-checkbox__icon {
  color: var(--sl-text-inverse, #ffffff);
}

.sl-checkbox__label {
  font-size: 0.875rem;
  color: var(--sl-text-secondary, #475569);
  line-height: 1;
}
</style>
