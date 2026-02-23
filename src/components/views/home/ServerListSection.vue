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
      <ServerCard v-for="server in servers" :key="server.id" :server="server" />
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
}

@media (max-width: 768px) {
  .server-grid {
    grid-template-columns: 1fr;
  }
}
</style>
