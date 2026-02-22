<script setup lang="ts">
import { watch, onUnmounted, ref, nextTick } from "vue";
import { X } from "lucide-vue-next";
import { i18n } from "@language";

interface Props {
  visible: boolean;
  title?: string;
  width?: string;
  closeOnOverlay?: boolean;
  autoClose?: number;
}

const props = withDefaults(defineProps<Props>(), {
  width: "480px",
  closeOnOverlay: true,
  autoClose: 0,
});

const emit = defineEmits<{
  close: [];
}>();

const handleClose = () => emit("close");

const modalRef = ref<HTMLElement | null>(null);
let previousActiveElement: Element | null = null;

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    handleClose();
  }
}

let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.visible,
  (newVisible) => {
    if (autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }

    if (newVisible) {
      previousActiveElement = document.activeElement;
      nextTick(() => {
        modalRef.value?.focus();
      });
      document.addEventListener("keydown", handleKeydown);

      if (props.autoClose > 0) {
        autoCloseTimer = setTimeout(() => {
          handleClose();
        }, props.autoClose);
      }
    } else {
      document.removeEventListener("keydown", handleKeydown);
      if (previousActiveElement instanceof HTMLElement) {
        previousActiveElement.focus();
      }
      previousActiveElement = null;
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  if (autoCloseTimer) {
    clearTimeout(autoCloseTimer);
  }
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="sl-modal-overlay" @click="closeOnOverlay && handleClose()">
      <div
        ref="modalRef"
        class="sl-modal glass-strong"
        :style="{ maxWidth: width }"
        tabindex="-1"
        role="dialog"
        aria-modal="true"
        @click.stop
      >
        <div class="sl-modal-header">
          <h3 v-if="title" class="sl-modal-title">{{ title }}</h3>
          <button
            class="sl-modal-close"
            @click="handleClose"
            :aria-label="i18n.t('common.close_modal')"
          >
            <X :size="18" />
          </button>
        </div>
        <div class="sl-modal-body">
          <slot />
        </div>
        <div v-if="$slots.footer" class="sl-modal-footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.sl-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: overlay-fade var(--sl-transition-fast) ease;
}

.sl-modal {
  width: 90%;
  max-width: 480px;
  border-radius: var(--sl-radius-lg, 8px);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15);
  background: var(--sl-surface, #fff);
  animation: modal-slide var(--sl-transition-normal) ease;
}

.sl-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--sl-border-light, #e5e7eb);
}

.sl-modal-title {
  font-size: 17px;
  font-weight: 600;
  margin: 0;
  color: var(--sl-text-primary, #1f2937);
}

.sl-modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-md, 6px);
  border: none;
  background: transparent;
  color: var(--sl-text-tertiary, #6b7280);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
  flex-shrink: 0;
}

.sl-modal-close:hover {
  background: var(--sl-bg-tertiary, #f3f4f6);
  color: var(--sl-text-primary, #1f2937);
}

.sl-modal-body {
  padding: 20px;
  max-height: 70vh;
  overflow-y: auto;
}

.sl-modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--sl-border-light, #e5e5e5);
}

@keyframes overlay-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes modal-slide {
  from {
    opacity: 0;
    transform: translateY(-10px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.sl-modal-overlay,
.sl-modal {
  will-change: transform, opacity;
  backface-visibility: hidden;
}

.glass-strong {
  backdrop-filter: blur(4px);
  background: var(--sl-surface, rgba(255, 255, 255, 0.95));
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(0, 0, 0, 0.3);
}

.sl-modal-title {
  color: var(--sl-text-primary);
}

.sl-modal-body {
  color: var(--sl-text-secondary);
}
</style>
