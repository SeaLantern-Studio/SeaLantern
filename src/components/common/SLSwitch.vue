<script setup lang="ts">
import { ref } from "vue";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Props {
  modelValue?: boolean;
  label?: string;
  disabled?: boolean;
  componentId?: string;
  size?: "sm" | "md" | "lg";
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  disabled: false,
  size: "md",
});

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const isAnimating = ref(false);

const handleClick = () => {
  if (!props.disabled) {
    isAnimating.value = true;
    emit("update:modelValue", !props.modelValue);
    setTimeout(() => {
      isAnimating.value = false;
    }, 400);
  }
};

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-switch-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLSwitch",
  get: (prop) => (prop === "value" ? props.modelValue : undefined),
  set: (prop, value) => {
    if (prop === "value") emit("update:modelValue", !!value);
  },
  call: () => undefined,
  on: (event, _cb) => {
    if (event === "change") {
    }
    return () => {};
  },
  el: () => elRef.value,
});
</script>

<template>
  <label
    ref="elRef"
    class="sl-switch-wrapper"
    :class="{ disabled: props.disabled, [`size-${props.size}`]: true }"
  >
    <div
      class="sl-switch"
      :class="{ active: props.modelValue, animating: isAnimating }"
      :tabindex="props.disabled ? -1 : 0"
      role="switch"
      :aria-checked="props.modelValue"
      :aria-disabled="props.disabled"
      @click="handleClick"
      @keydown.enter.prevent="handleClick"
      @keydown.space.prevent="handleClick"
    >
      <div class="sl-switch-track">
        <div class="track-glow" />
      </div>
      <div class="sl-switch-thumb" />
      <div class="sl-switch-ripple" v-if="isAnimating" />
    </div>
    <span v-if="props.label" class="sl-switch-label">{{ props.label }}</span>
  </label>
</template>

<style scoped>
.sl-switch-wrapper {
  display: inline-flex;
  align-items: center;
  user-select: none;
}

.sl-switch-wrapper:not(.disabled) {
  cursor: pointer;
}

.sl-switch-wrapper.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  filter: grayscale(0.3);
}

.sl-switch-wrapper > * + * {
  margin-left: var(--sl-space-sm, 8px);
}

.sl-switch {
  --switch-width: 44px;
  --switch-height: 24px;
  --thumb-size: 20px;
  --thumb-offset: 2px;
  --thumb-translate: 20px;

  position: relative;
  width: var(--switch-width);
  height: var(--switch-height);
  border-radius: var(--sl-radius-full, 9999px);
  flex-shrink: 0;
  outline: none;
  -webkit-tap-highlight-color: transparent;
}

.sl-switch:focus-visible {
  outline: 2px solid var(--sl-primary, #0ea5e9);
  outline-offset: 3px;
}

.sl-switch-track {
  position: absolute;
  inset: 0;
  background: var(--sl-bg-tertiary, #e2e8f0);
  border-radius: inherit;
  transition: background 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.track-glow {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, var(--sl-primary, #0ea5e9), var(--sl-accent, #06b6d4));
  opacity: 0;
  transition: opacity 0.3s ease;
}

.sl-switch:hover:not(.active) .sl-switch-track {
  background: var(--sl-border, #cbd5e1);
}

.sl-switch.active .sl-switch-track {
  background: var(--sl-primary, #0ea5e9);
}

.sl-switch.active .track-glow {
  opacity: 0.15;
}

.sl-switch:hover.active .sl-switch-track {
  background: var(--sl-primary-dark, #0369a1);
}

.sl-switch-thumb {
  position: absolute;
  top: var(--thumb-offset);
  left: var(--thumb-offset);
  width: var(--thumb-size);
  height: var(--thumb-size);
  background: var(--sl-surface, #ffffff);
  border-radius: 50%;
  box-shadow:
    0 2px 4px rgba(0, 0, 0, 0.1),
    0 4px 8px rgba(0, 0, 0, 0.08);
  transition:
    transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1),
    box-shadow 0.3s ease,
    background 0.3s ease;
  will-change: transform;
  z-index: 1;
}

.sl-switch:hover .sl-switch-thumb {
  box-shadow:
    0 4px 8px rgba(0, 0, 0, 0.12),
    0 8px 16px rgba(0, 0, 0, 0.08);
}

.sl-switch:active .sl-switch-thumb {
  box-shadow:
    0 1px 2px rgba(0, 0, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.06);
}

.sl-switch.active .sl-switch-thumb {
  transform: translateX(var(--thumb-translate));
  background: var(--sl-surface, #ffffff);
}

.sl-switch-ripple {
  position: absolute;
  inset: -6px;
  border-radius: inherit;
  background: var(--sl-primary, #0ea5e9);
  opacity: 0;
  animation: switch-ripple 0.4s ease-out;
  pointer-events: none;
}

@keyframes switch-ripple {
  0% {
    opacity: 0.25;
    transform: scale(0.9);
  }
  100% {
    opacity: 0;
    transform: scale(1.15);
  }
}

.sl-switch.animating .sl-switch-thumb {
  animation: thumb-press 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes thumb-press {
  0% {
    transform: scale(1);
  }
  30% {
    transform: scale(0.85);
  }
  60% {
    transform: scale(1.05);
  }
  100% {
    transform: scale(1);
  }
}

.sl-switch.active.animating .sl-switch-thumb {
  animation: thumb-press-active 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes thumb-press-active {
  0% {
    transform: translateX(var(--thumb-translate)) scale(1);
  }
  30% {
    transform: translateX(var(--thumb-translate)) scale(0.85);
  }
  60% {
    transform: translateX(var(--thumb-translate)) scale(1.05);
  }
  100% {
    transform: translateX(var(--thumb-translate)) scale(1);
  }
}

.sl-switch-label {
  font-size: 0.875rem;
  color: var(--sl-text-secondary, #475569);
  line-height: 1;
  transition: color 0.2s ease;
}

.sl-switch-wrapper:hover .sl-switch-label {
  color: var(--sl-text-primary, #0f172a);
}

/* Size variants */
.sl-switch-wrapper.size-sm .sl-switch {
  --switch-width: 36px;
  --switch-height: 20px;
  --thumb-size: 16px;
  --thumb-translate: 16px;
}

.sl-switch-wrapper.size-lg .sl-switch {
  --switch-width: 52px;
  --switch-height: 28px;
  --thumb-size: 24px;
  --thumb-translate: 24px;
}

.sl-switch-wrapper.size-sm .sl-switch-label {
  font-size: 0.8125rem;
}

.sl-switch-wrapper.size-lg .sl-switch-label {
  font-size: 0.9375rem;
}

/* Dark mode adjustments */
[data-theme="dark"] .sl-switch-track {
  background: var(--sl-bg-tertiary, #242836);
}

[data-theme="dark"] .sl-switch:hover:not(.active) .sl-switch-track {
  background: var(--sl-border, rgba(255, 255, 255, 0.15));
}

[data-theme="dark"] .sl-switch-thumb {
  background: var(--sl-bg-secondary, #1a1d28);
  box-shadow:
    0 2px 4px rgba(0, 0, 0, 0.3),
    0 4px 8px rgba(0, 0, 0, 0.2);
}

[data-theme="dark"] .sl-switch:hover .sl-switch-thumb {
  box-shadow:
    0 4px 8px rgba(0, 0, 0, 0.4),
    0 8px 16px rgba(0, 0, 0, 0.3);
}

[data-theme="dark"] .sl-switch.active .sl-switch-thumb {
  background: var(--sl-surface, #1e2130);
}
</style>
