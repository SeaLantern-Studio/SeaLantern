<script setup lang="ts">
interface Props {
  status: "running" | "starting" | "stopping" | "stopped";
  label: string;
}

defineProps<Props>();
</script>

<template>
  <span class="status-indicator" :class="status">
    <span class="status-dot"></span>
    <span class="status-label">{{ label }}</span>
  </span>
</template>

<style scoped>
.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 4px 12px;
  border-radius: var(--sl-radius-full);
  font-size: var(--sl-font-size-xs);
  font-weight: 500;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-indicator.running {
  background: rgba(34, 197, 94, 0.1);
  color: var(--sl-success);
}

.status-indicator.running .status-dot {
  background: var(--sl-success);
}

.status-indicator.stopped {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
}

.status-indicator.stopped .status-dot {
  background: var(--sl-text-tertiary);
}

.status-indicator.starting,
.status-indicator.stopping {
  background: rgba(245, 158, 11, 0.1);
  color: var(--sl-warning);
}

.status-indicator.starting .status-dot,
.status-indicator.stopping .status-dot {
  background: var(--sl-warning);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}
</style>
