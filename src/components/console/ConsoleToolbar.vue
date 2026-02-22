<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  serverId: string;
  serverName: string;
  statusClass: string;
  statusText: string;
  isRunning: boolean;
  isStopped: boolean;
  isStopping: boolean;
  startLoading: boolean;
  stopLoading: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "start"): void;
  (e: "stop"): void;
  (e: "export"): void;
  (e: "clear"): void;
}>();
</script>

<template>
  <div class="console-toolbar">
    <div class="toolbar-left">
      <div v-if="serverId" class="server-name-display">
        {{ serverName || i18n.t("common.console") }}
      </div>
      <div v-else class="server-name-display">
        {{ i18n.t("home.no_servers") }}
      </div>
      <div v-if="serverId" class="status-indicator" :class="statusClass">
        <span class="status-dot"></span>
        <span class="status-label">{{ statusText }}</span>
      </div>
    </div>
    <div class="toolbar-right">
      <SLButton
        variant="primary"
        size="sm"
        :loading="startLoading"
        :disabled="isRunning || isStopping || startLoading"
        @click="emit('start')"
        >{{ i18n.t("home.start") }}</SLButton
      >
      <SLButton
        variant="danger"
        size="sm"
        :loading="stopLoading"
        :disabled="isStopped || isStopping || stopLoading"
        @click="emit('stop')"
        >{{ i18n.t("home.stop") }}</SLButton
      >
      <SLButton variant="secondary" size="sm" @click="emit('export')">{{
        i18n.t("console.copy_log")
      }}</SLButton>
      <SLButton variant="ghost" size="sm" @click="emit('clear')">{{
        i18n.t("console.clear_log")
      }}</SLButton>
    </div>
  </div>
</template>

<style scoped>
.console-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
}
.toolbar-right {
  display: flex;
  gap: var(--sl-space-xs);
}
.server-name-display {
  font-weight: 600;
}
.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 2px 10px;
  border-radius: var(--sl-radius-full);
  font-size: 0.8125rem;
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
}
</style>
