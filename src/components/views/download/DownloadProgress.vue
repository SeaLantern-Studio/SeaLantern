<script setup lang="ts">
import { SLProgress } from "@components/common";
import { i18n } from "@language";
import { formatBytes } from "@utils/serverUtils";

const props = defineProps<{
  taskInfo: {
    progress: number;
    downloaded: number;
    totalSize: number;
    isFinished: boolean;
  };
  taskError: string | null;
  statusLabel: string;
}>();
</script>

<template>
  <div class="progress-wrapper">
    <SLProgress
      :value="taskInfo.progress"
      :variant="taskError ? 'error' : taskInfo.isFinished ? 'success' : 'primary'"
      :label="statusLabel"
    />
    <div class="progress-footer">
      <span class="size-text"
        >{{ formatBytes(taskInfo.downloaded) }} / {{ formatBytes(taskInfo.totalSize) }}</span
      >
      <span class="percent-text">{{ taskInfo.progress.toFixed(1) }}%</span>
    </div>
  </div>
</template>

<style scoped>
.progress-wrapper {
  width: 100%;
  max-width: 560px; /* 略窄于卡片，更有层次感 */
  background: var(--sl-bg-secondary, #f9f9f9); /* 可选：给个淡淡的底色背景 */
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border-light, #eee);
}

.progress-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-secondary);
  font-family: var(--sl-font-mono, monospace), serif;
}
</style>
