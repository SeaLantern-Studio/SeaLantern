<script setup lang="ts">
import { i18n } from "@language";
import { recentAlerts } from "@utils/serverUtils";
</script>

<template>
  <div v-if="recentAlerts.length > 0" class="alerts-section">
    <h3 class="section-title">{{ i18n.t("home.recent_alerts") }}</h3>
    <div class="alerts-list">
      <div
        v-for="(alert, i) in recentAlerts"
        :key="i"
        class="alert-item"
        :class="{
          'alert-error': alert.line.includes('ERROR') || alert.line.includes('FATAL'),
          'alert-warn': alert.line.includes('WARN'),
        }"
      >
        <span class="alert-server">{{ alert.server }}</span>
        <span class="alert-text">{{ alert.line }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.alerts-section {
  margin-top: var(--sl-space-sm);
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 1.0625rem;
  font-weight: 600;
  margin-bottom: var(--sl-space-sm);
}

.alerts-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 200px;
  overflow-y: auto;
  background: #1e1e2e;
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.alert-item {
  display: flex;
  gap: var(--sl-space-sm);
  font-family: var(--sl-font-mono);
  font-size: 0.75rem;
  line-height: 1.6;
  color: #cdd6f4;
}

.alert-error {
  color: #f38ba8;
}

.alert-warn {
  color: #fab387;
}

.alert-server {
  flex-shrink: 0;
  padding: 0 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--sl-radius-sm);
  color: #89b4fa;
}

.alert-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
