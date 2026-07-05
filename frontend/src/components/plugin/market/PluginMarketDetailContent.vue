<script setup lang="ts">
import { i18n } from "@language";
import type { MarketPermissionLevel, MarketPlugin } from "./pluginMarketShared";
import type { MarketPluginInfo } from "@api/plugin";

defineProps<{
  plugin: MarketPlugin;
  pluginDetail: MarketPluginInfo | null;
  resolveI18n: (value: Record<string, string> | string | undefined) => string;
  getPermissionLevel: (perm: string) => MarketPermissionLevel;
  getPermissionLabel: (perm: string) => string;
  getPermissionDesc: (perm: string) => string;
}>();
</script>

<template>
  <div class="detail-body">
    <p class="detail-desc">
      {{ resolveI18n(pluginDetail?.description || plugin.description) }}
    </p>

    <div v-if="pluginDetail?.permissions?.length" class="detail-section">
      <h3>{{ i18n.t("market.permissions") }}</h3>
      <div class="permission-badges">
        <span
          v-for="perm in pluginDetail.permissions"
          :key="perm"
          :class="['perm-badge', `perm-badge--${getPermissionLevel(perm)}`]"
          :title="getPermissionDesc(perm)"
        >
          {{ getPermissionLabel(perm) }}
        </span>
      </div>
    </div>

    <div v-if="pluginDetail?.changelog" class="detail-section">
      <h3>{{ i18n.t("market.changelog") }}</h3>
      <pre class="changelog">{{ pluginDetail.changelog }}</pre>
    </div>
  </div>
</template>

<style scoped>
.detail-body {
  margin-bottom: 20px;
}

.detail-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.detail-section {
  margin-top: 16px;
}

.detail-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px;
}

.permission-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-badge {
  padding: 3px 10px;
  border-radius: var(--sl-radius-lg);
  font-size: 12px;
  font-weight: 500;
  cursor: default;
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  border: 1px solid var(--sl-border);
}

.perm-badge--dangerous {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.perm-badge--critical {
  background: rgba(239, 68, 68, 0.12);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.changelog {
  margin: 0;
  padding: 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-md);
  font-size: 12px;
  color: var(--sl-text-secondary);
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}
</style>
