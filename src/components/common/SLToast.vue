<script setup lang="ts">
import { AlertTriangle, Check, Info, X } from "lucide-vue-next";
import { useToast } from "../../composables/useToast";
import type { ToastType } from "../../composables/useToast";

const { toasts, removeToast } = useToast();

const iconComponents: Record<ToastType, typeof Check> = {
  success: Check,
  error: X,
  warning: AlertTriangle,
  info: Info,
};
</script>

<template>
  <Teleport to="body">
    <div class="sl-toast-container" aria-live="polite">
      <TransitionGroup name="sl-toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="sl-toast"
          :class="`sl-toast--${toast.type}`"
          role="alert"
        >
          <component
            :is="iconComponents[toast.type] || iconComponents.info"
            class="sl-toast__icon"
            :size="18"
            aria-hidden="true"
          />
          <span class="sl-toast__message">{{ toast.message }}</span>
          <button class="sl-toast__close" @click="removeToast(toast.id)" aria-label="close">
            <X :size="14" aria-hidden="true" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style>
.sl-toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 99999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
  max-width: 380px;
}

.sl-toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-radius: var(--sl-radius-md, 10px);
  background: var(--sl-surface, #ffffff);
  border: 1px solid var(--sl-border, #e2e8f0);
  box-shadow: var(--sl-shadow-lg, 0 8px 24px rgba(0, 0, 0, 0.08));
  backdrop-filter: blur(12px);
  pointer-events: auto;
  font-size: 0.875rem;
  color: var(--sl-text-primary, #0f172a);
}

.sl-toast--success {
  border-left: 3px solid var(--sl-success, #22c55e);
}

.sl-toast--success .sl-toast__icon {
  color: var(--sl-success, #22c55e);
}

.sl-toast--error {
  border-left: 3px solid var(--sl-error, #ef4444);
}

.sl-toast--error .sl-toast__icon {
  color: var(--sl-error, #ef4444);
}

.sl-toast--warning {
  border-left: 3px solid var(--sl-warning, #f59e0b);
}

.sl-toast--warning .sl-toast__icon {
  color: var(--sl-warning, #f59e0b);
}

.sl-toast--info {
  border-left: 3px solid var(--sl-info, #3b82f6);
}

.sl-toast--info .sl-toast__icon {
  color: var(--sl-info, #3b82f6);
}

.sl-toast__icon {
  flex-shrink: 0;
}

.sl-toast__message {
  flex: 1;
  line-height: 1.4;
}

.sl-toast__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: var(--sl-radius-sm, 6px);
  background: transparent;
  color: var(--sl-text-tertiary, #94a3b8);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    background-color var(--sl-transition-fast, 0.15s ease),
    color var(--sl-transition-fast, 0.15s ease);
}

.sl-toast__close:hover {
  background: var(--sl-bg-tertiary, #e2e8f0);
  color: var(--sl-text-primary, #0f172a);
}

/* -- Transition -- */
.sl-toast-enter-active,
.sl-toast-leave-active {
  transition: all 0.3s ease;
}

.sl-toast-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.sl-toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}

.sl-toast-move {
  transition: transform 0.3s ease;
}
</style>
