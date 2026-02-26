<script setup lang="ts">
import { Server } from "lucide-vue-next";
import ServerCard from "./ServerCard.vue";
import type { ServerInstance } from "@type/server";
import { i18n } from "@language";

defineProps<{
  servers: ServerInstance[];
  loading: boolean;
}>();
</script>

<template>
  <div class="server-list-section">
    <div class="section-header">
      <h3 class="section-title">
        {{ i18n.t("home.title") }}
        <span class="server-count">{{ servers.length }}</span>
      </h3>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("common.loading") }}</span>
    </div>

    <div v-else-if="servers.length === 0" class="empty-state">
      <Server :size="64" :stroke-width="1" class="empty-icon" />
      <p class="text-body">{{ i18n.t("home.no_servers") }}</p>
      <p class="text-caption">{{ i18n.t("home.create_first") }}</p>
    </div>

    <div v-else class="server-grid">
      <TransitionGroup name="server-list">
        <ServerCard v-for="server in servers" :key="server.id" :server="server" />
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.server-list-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-title {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 1.0625rem;
  font-weight: 600;
}

.server-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-radius: var(--sl-radius-full);
  font-size: 0.75rem;
  font-weight: 600;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
  gap: var(--sl-space-sm);
}

.empty-icon {
  color: var(--sl-text-tertiary);
  opacity: 0.5;
}

.text-body {
  font-size: 0.9375rem;
  color: var(--sl-text-primary);
}

.text-caption {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
}

.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: var(--sl-space-lg);
  align-items: stretch;
}

/* 服务器列表过渡动画 */
.server-list-enter-active {
  animation: server-enter 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.server-list-leave-active {
  animation: server-leave 0.25s ease;
  position: absolute;
}

.server-list-move {
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes server-enter {
  from {
    opacity: 0;
    transform: translateY(-16px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes server-leave {
  from {
    opacity: 1;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(0.9);
  }
}

@media (max-width: 768px) {
  .server-grid {
    grid-template-columns: 1fr;
  }
}
</style>
