<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { SystemInfo } from "@api/system";
import { formatBytes } from "@utils/formatters";

const props = defineProps<{
  systemInfo: SystemInfo | null;
  version: string;
  loading: boolean;
  error: string | null;
  memoryDisplayPrecision: number;
}>();

const emit = defineEmits<{
  (e: "refresh"): void;
}>();

function normalizePrecision(value: number): number {
  return value === 0 || value === 2 || value === 4 ? value : 2;
}

function formatBytesToGb(value: number, precision: number): string {
  const normalizedPrecision = normalizePrecision(precision);
  const gb = value / 1024 / 1024 / 1024;
  return `${gb.toFixed(normalizedPrecision)} GB`;
}

function formatBytesWithPrecision(value: number, precision: number): string {
  const normalizedPrecision = normalizePrecision(precision);
  if (value <= 0) {
    return formatBytes(0);
  }

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const base = 1024;
  const unitIndex = Math.min(Math.floor(Math.log(value) / Math.log(base)), units.length - 1);
  const scaled = value / Math.pow(base, unitIndex);
  return `${scaled.toFixed(normalizedPrecision)} ${units[unitIndex]}`;
}

function formatUptime(totalSeconds: number): string {
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const parts: string[] = [];

  if (days > 0) {
    parts.push(`${days}d`);
  }
  if (hours > 0 || days > 0) {
    parts.push(`${hours}h`);
  }
  parts.push(`${minutes}m`);
  return parts.join(" ");
}

const items = computed(() => {
  if (!props.systemInfo) return [];

  const precision = normalizePrecision(props.memoryDisplayPrecision);

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
      value: `${formatBytesToGb(props.systemInfo.memory.used, precision)} / ${formatBytesToGb(props.systemInfo.memory.total, precision)}`,
      detail: [
        `${i18n.t("developer.server_instances_memory")}: ${formatBytesWithPrecision(props.systemInfo.memory.server_instances_used, precision)}`,
        `${i18n.t("developer.app_memory")}: ${formatBytesWithPrecision(props.systemInfo.memory.app_used, precision)}`,
      ],
    },
    { label: i18n.t("developer.process_count"), value: String(props.systemInfo.process_count) },
    { label: i18n.t("developer.uptime"), value: formatUptime(props.systemInfo.uptime) },
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
        <span v-for="detail in item.detail || []" :key="detail" class="developer-system-detail">
          {{ detail }}
        </span>
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

.developer-system-detail {
  color: var(--sl-text-secondary);
  font-size: 12px;
  line-height: 1.5;
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
