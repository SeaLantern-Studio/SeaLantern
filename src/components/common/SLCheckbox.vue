<script setup lang="ts">
import { ref } from "vue";
import { Check, Minus } from "lucide-vue-next";
import { useRegisterComponent } from "@composables/useRegisterComponent";

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

const isAnimating = ref(false);

const handleChange = () => {
  if (!props.disabled) {
    isAnimating.value = true;
    emit("update:modelValue", !props.modelValue);
    setTimeout(() => {
      isAnimating.value = false;
    }, 300);
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
        'sl-checkbox__box--animating': isAnimating,
      }"
      @click="handleChange"
      @keydown.space.prevent="handleChange"
      role="checkbox"
      :aria-checked="indeterminate ? 'mixed' : modelValue"
      :aria-disabled="disabled"
      tabindex="0"
    >
      <div class="sl-checkbox__ripple" v-if="isAnimating" />
      <Check
        v-if="modelValue"
        class="sl-checkbox__icon sl-checkbox__icon--check"
        :size="12"
        :stroke-width="3"
        aria-hidden="true"
      />
      <Minus
        v-else-if="indeterminate"
        class="sl-checkbox__icon sl-checkbox__icon--minus"
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
  opacity: 0.4;
  cursor: not-allowed;
  filter: grayscale(0.3);
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
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sl-checkbox__box:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px var(--sl-primary-bg, rgba(14, 165, 233, 0.15));
  border-color: var(--sl-primary, #0ea5e9);
}

.sl-checkbox:not(.sl-checkbox--disabled) .sl-checkbox__box:hover {
  border-color: var(--sl-primary, #0ea5e9);
  transform: scale(1.05);
}

.sl-checkbox:not(.sl-checkbox--disabled) .sl-checkbox__box:active {
  transform: scale(0.95);
}

.sl-checkbox__box--checked,
.sl-checkbox__box--indeterminate {
  background: var(--sl-primary, #0ea5e9);
  border-color: var(--sl-primary, #0ea5e9);
}

.sl-checkbox__box--animating {
  animation: checkbox-pop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes checkbox-pop {
  0% {
    transform: scale(1);
  }
  40% {
    transform: scale(0.85);
  }
  70% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
  }
}

.sl-checkbox__ripple {
  position: absolute;
  inset: -8px;
  border-radius: 50%;
  background: var(--sl-primary, #0ea5e9);
  opacity: 0;
  animation: checkbox-ripple 0.4s ease-out;
  pointer-events: none;
}

@keyframes checkbox-ripple {
  0% {
    opacity: 0.3;
    transform: scale(0.5);
  }
  100% {
    opacity: 0;
    transform: scale(1.5);
  }
}

.sl-checkbox__icon {
  color: var(--sl-text-inverse, #ffffff);
  position: relative;
  z-index: 1;
}

.sl-checkbox__icon--check {
  animation: icon-check-in 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sl-checkbox__icon--minus {
  animation: icon-fade-in 0.2s ease;
}

@keyframes icon-check-in {
  0% {
    opacity: 0;
    transform: scale(0.5) rotate(-45deg);
  }
  100% {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
}

@keyframes icon-fade-in {
  0% {
    opacity: 0;
    transform: scale(0.8);
  }
  100% {
    opacity: 1;
    transform: scale(1);
  }
}

.sl-checkbox__label {
  font-size: 0.875rem;
  color: var(--sl-text-secondary, #475569);
  line-height: 1;
  transition: color 0.2s ease;
}

.sl-checkbox:not(.sl-checkbox--disabled):hover .sl-checkbox__label {
  color: var(--sl-text-primary, #0f172a);
}

/* Dark mode */
[data-theme="dark"] .sl-checkbox__box {
  background: var(--sl-bg-tertiary, #242836);
  border-color: var(--sl-border, rgba(255, 255, 255, 0.1));
}

[data-theme="dark"] .sl-checkbox__box--checked,
[data-theme="dark"] .sl-checkbox__box--indeterminate {
  background: var(--sl-primary, #60a5fa);
  border-color: var(--sl-primary, #60a5fa);
}
</style>
