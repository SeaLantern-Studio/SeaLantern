<script setup lang="ts">
import { ref } from "vue";
import { Loader2 } from "lucide-vue-next";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Props {
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  loading?: boolean;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
  size: "md",
  disabled: false,
  loading: false,
});

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-button-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLButton",
  get: (prop) => (prop === "disabled" ? props.disabled : undefined),
  set: () => {},
  call: (method) => {
    if (method === "click") elRef.value?.click();
  },
  on: () => () => {},
  el: () => elRef.value,
});
</script>

<template>
  <button
    ref="elRef"
    class="sl-button"
    :class="[
      `sl-button--${variant}`,
      `sl-button--${size}`,
      { 'sl-button--disabled': disabled || loading },
    ]"
    :disabled="disabled || loading"
    :aria-busy="loading"
  >
    <Loader2 v-if="loading" class="sl-button-spinner" :size="16" />
    <slot />
  </button>
</template>

<style scoped>
.sl-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-xs, 0.5rem);
  font-weight: 500;
  border-radius: var(--sl-radius-md, 0.375rem);
  border: 1px solid transparent;
  transition:
    background-color var(--sl-transition-fast) ease-in-out,
    border-color var(--sl-transition-fast) ease-in-out,
    box-shadow var(--sl-transition-fast) ease-in-out,
    transform 0.1s ease-in-out;
  cursor: pointer;
  white-space: nowrap;
  user-select: none;
  -webkit-tap-highlight-color: transparent;
  will-change: transform, box-shadow;
}

.sl-button--sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  line-height: 1.25;
}

.sl-button--md {
  padding: 0.5rem 1.25rem;
  font-size: 0.875rem;
  line-height: 1.375;
}

.sl-button--lg {
  padding: 0.75rem 1.75rem;
  font-size: 1rem;
  line-height: 1.5;
}

.sl-button--primary {
  background-color: var(--sl-primary, #2563eb);
  color: var(--sl-text-inverse, #ffffff);
  box-shadow: 0 1px 3px rgba(37, 99, 235, 0.3);
}

.sl-button--primary:hover:not(:disabled) {
  background-color: var(--sl-primary-dark, #1d4ed8);
  box-shadow: 0 2px 5px rgba(37, 99, 235, 0.4);
  transform: translateY(-1px);
}

.sl-button--primary:active:not(:disabled) {
  transform: translateY(0);
}

.sl-button--secondary {
  background-color: var(--sl-bg-secondary, #f8fafc);
  color: var(--sl-text-primary, #1e293b);
  border-color: var(--sl-border, #cbd5e1);
}

.sl-button--secondary:hover:not(:disabled) {
  background-color: var(--sl-bg-tertiary, #f1f5f9);
  border-color: var(--sl-primary-light, #93c5fd);
}

.sl-button--ghost {
  background-color: transparent;
  color: var(--sl-text-secondary, #64748b);
}

.sl-button--ghost:hover:not(:disabled) {
  background-color: var(--sl-primary-bg, #eff6ff);
  color: var(--sl-primary, #2563eb);
}

.sl-button--danger {
  background-color: var(--sl-error, #dc2626);
  color: var(--sl-text-inverse, #ffffff);
  box-shadow: 0 1px 3px rgba(220, 38, 38, 0.3);
}

.sl-button--danger:hover:not(:disabled) {
  background-color: #b91c1c;
  box-shadow: 0 2px 5px rgba(220, 38, 38, 0.4);
  transform: translateY(-1px);
}

.sl-button--success {
  background: var(--sl-success);
  color: var(--sl-text-inverse);
}

.sl-button--success:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

.sl-button--disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none !important;
  box-shadow: none !important;
}

.sl-button-spinner {
  flex-shrink: 0;
  animation: spin 0.75s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
