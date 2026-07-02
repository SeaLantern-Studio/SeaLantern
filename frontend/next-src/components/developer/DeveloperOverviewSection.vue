<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import WorkbenchFactGrid from "@next-src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import type { SystemInfo } from "@api/system";
import { i18n } from "@language";
import { formatBytes } from "@utils/formatters";

interface Props {
  version: string;
  systemInfo: SystemInfo | null;
  loading: boolean;
  error: string | null;
  memoryDisplayPrecision: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  refresh: [];
  copySystem: [];
}>();

function normalizePrecision(value: number): number {
  return value === 0 || value === 2 || value === 4 ? value : 2;
}

function formatMemory(value: number): string {
  const precision = normalizePrecision(props.memoryDisplayPrecision);
  if (value <= 0) {
    return formatBytes(0);
  }
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const unitIndex = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  return `${(value / 1024 ** unitIndex).toFixed(precision)} ${units[unitIndex]}`;
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

const factItems = computed(() => {
  if (!props.systemInfo) {
    return [];
  }

  return [
    { label: i18n.t("developer.app_version"), value: props.version },
    { label: i18n.t("developer.os"), value: `${props.systemInfo.os_name} ${props.systemInfo.os_version}` },
    { label: i18n.t("developer.kernel"), value: props.systemInfo.kernel_version },
    { label: i18n.t("developer.host"), value: props.systemInfo.host_name || "-" },
    { label: i18n.t("developer.cpu"), value: `${props.systemInfo.cpu.name} (${props.systemInfo.cpu.count})` },
    { label: i18n.t("developer.memory"), value: `${formatMemory(props.systemInfo.memory.used)} / ${formatMemory(props.systemInfo.memory.total)}` },
    { label: i18n.t("developer.server_instances_memory"), value: formatMemory(props.systemInfo.memory.server_instances_used) },
    { label: i18n.t("developer.app_memory"), value: formatMemory(props.systemInfo.memory.app_used) },
    { label: i18n.t("developer.process_count"), value: String(props.systemInfo.process_count) },
    { label: i18n.t("developer.uptime"), value: formatUptime(props.systemInfo.uptime) },
  ];
});
</script>

<template>
  <WorkbenchPanel :title="i18n.t('developer.next.overview.title')" :description="i18n.t('developer.next.overview.description')">
    <template #actions>
      <SLButton variant="secondary" size="sm" @click="emit('refresh')">{{ i18n.t("common.refresh") }}</SLButton>
      <SLButton variant="secondary" size="sm" :disabled="!systemInfo" @click="emit('copySystem')">{{ i18n.t("developer.copy_system") }}</SLButton>
    </template>

    <p v-if="error" class="developer-overview-section__error">{{ error }}</p>
    <p v-else-if="loading && !systemInfo" class="developer-overview-section__muted">{{ i18n.t("common.loading") }}</p>
    <p v-else-if="!systemInfo" class="developer-overview-section__muted">{{ i18n.t("developer.system_empty") }}</p>
    <WorkbenchFactGrid v-else :items="factItems" />
  </WorkbenchPanel>
</template>

<style scoped>
.developer-overview-section__muted,
.developer-overview-section__error { margin: 0; font-size: 0.84rem; line-height: 1.45; }
.developer-overview-section__muted { color: var(--sl-text-secondary); }
.developer-overview-section__error { color: var(--sl-error); }
</style>
