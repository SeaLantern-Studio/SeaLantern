<script setup lang="ts">
interface Props {
  visible: boolean;
  title?: string;
  width?: string;
  closeOnOverlay?: boolean;
}

withDefaults(defineProps<Props>(), {
  width: "480px",
  closeOnOverlay: true,
});

const emit = defineEmits<{
  close: [];
}>();

const handleClose = () => emit("close");
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="sl-modal-overlay" @click="closeOnOverlay && handleClose()">
      <div class="sl-modal glass-strong" :style="{ maxWidth: width }" @click.stop>
        <div class="sl-modal-header">
          <h3 v-if="title" class="sl-modal-title">{{ title }}</h3>
          <button class="sl-modal-close" @click="handleClose" aria-label="关闭弹窗">
            <svg width="18" height="18" viewBox="0 0 24 24">
              <path d="M18 6 6 18 M6 6l12 12" stroke="currentColor" stroke-width="2" />
            </svg>
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
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: overlay-fade 0.2s ease;
}

.sl-modal {
  width: 90%;
  max-width: 480px;
  border-radius: var(--sl-radius-lg, 8px);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15);
  background: var(--sl-surface, #fff);
  animation: modal-slide 0.25s ease;
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
  transition: background-color 0.15s ease, color 0.15s ease;
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
  from { opacity: 0; }
  to { opacity: 1; }
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

/* 性能优化：减少复合图层 */
.sl-modal-overlay,
.sl-modal {
  will-change: transform, opacity;
  backface-visibility: hidden;
}

/* 毛玻璃效果可选，默认关闭以节省性能 */
.glass-strong {
  backdrop-filter: blur(4px);
  background: rgba(255, 255, 255, 0.95);
}
</style>