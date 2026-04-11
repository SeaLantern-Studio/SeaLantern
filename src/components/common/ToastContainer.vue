<script setup lang="ts">
import { useGlobalMessage } from "@composables/useMessage";

const { messages, remove } = useGlobalMessage();
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="toast"
          :class="`toast-${msg.type}`"
          @click="remove(msg.id)"
        >
          <span class="toast-message">{{ msg.message }}</span>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 54px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: var(--sl-radius-md);
  font-size: 0.8125rem;
  cursor: pointer;
  pointer-events: auto;
  max-width: 320px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.15);
}

.toast-error {
  background: rgba(239, 68, 68, 0.85);
  color: white;
}

.toast-success {
  background: rgba(34, 197, 94, 0.85);
  color: white;
}

.toast-warning {
  background: rgba(245, 158, 11, 0.85);
  color: white;
}

.toast-info {
  background: rgba(59, 130, 246, 0.85);
  color: white;
}

.toast-message {
  flex: 1;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
