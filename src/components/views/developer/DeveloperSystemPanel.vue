<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { SystemInfo } from "@api/system";

const props = defineProps<{
  systemInfo: SystemInfo | null;
  version: string;
  loading: boolean;
  error: string | null;
}>();

const emit = defineEmits<{
  (e: "refresh"): void;
}>();

const items = computed(() => {
  if (!props.systemInfo) return [];

  return [
    { label: i18n.t("developer.app_version"), value: props.version },
    {
      label: i18n.t("developer.os"),
      value: `${props.systemInfo.os_name} ${props.systemInfo.os_version}`,
    },
    { label: i18n.t("developer.kernel"), value: props.systemInfo.kernel_version },
    { label: i18n.t("developer.host"), value: props.systemInfo.host_name || "-" },
    {
      label: i18n.t("developer.cpu"),
      value: `${props.systemInfo.cpu.name} (${props.systemInfo.cpu.count})`,
    },
    {
      label: i18n.t("developer.memory"),
      value: `${Math.round(props.systemInfo.memory.used / 1024 / 1024 / 1024)} / ${Math.round(props.systemInfo.memory.total / 1024 / 1024 / 1024)} GB`,
    },
    { label: i18n.t("developer.process_count"), value: String(props.systemInfo.process_count) },
    { label: i18n.t("developer.uptime"), value: `${Math.floor(props.systemInfo.uptime / 60)} min` },
  ];
});
</script>

<template>
  <SLCard :title="i18n.t('developer.system_title')" :subtitle="i18n.t('developer.system_desc')">
    <template #actions>
      <SLButton variant="secondary" size="sm" @click="emit('refresh')">
        {{ i18n.t("common.refresh") }}
      </SLButton>
    </template>

    <p v-if="error" class="developer-panel-error">{{ error }}</p>
    <p v-else-if="loading && !systemInfo" class="developer-panel-muted">
      {{ i18n.t("common.loading") }}
    </p>
    <div v-else-if="systemInfo" class="developer-system-grid">
      <div v-for="item in items" :key="item.label" class="developer-system-item">
        <span class="developer-system-label">{{ item.label }}</span>
        <span class="developer-system-value">{{ item.value }}</span>
      </div>
    </div>
    <p v-else class="developer-panel-muted">{{ i18n.t("developer.system_empty") }}</p>
  </SLCard>
</template>

<style scoped>
.developer-system-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--sl-space-md);
}

.developer-system-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
}

.developer-system-label {
  font-size: 12px;
  color: var(--sl-text-tertiary);
}

.developer-system-value {
  color: var(--sl-text-primary);
  word-break: break-word;
}

.developer-panel-muted {
  margin: 0;
  color: var(--sl-text-tertiary);
}

.developer-panel-error {
  margin: 0;
  color: var(--sl-error);
}
</style>
