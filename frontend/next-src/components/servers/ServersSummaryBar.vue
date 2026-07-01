<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  totalCount: number;
  runningCount: number;
  refreshing: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  create: [];
  importExisting: [];
  refresh: [];
}>();
</script>

<template>
  <section class="servers-summary-bar">
    <div class="servers-summary-bar__metrics">
      <div class="servers-summary-bar__metric-card">
        <span class="servers-summary-bar__metric-label">{{
          i18n.t("servers.next.summary.total_label")
        }}</span>
        <strong class="servers-summary-bar__metric-value">{{ totalCount }}</strong>
      </div>
      <div class="servers-summary-bar__metric-card servers-summary-bar__metric-card--running">
        <span class="servers-summary-bar__metric-label">{{
          i18n.t("servers.next.summary.running_label")
        }}</span>
        <strong class="servers-summary-bar__metric-value">{{ runningCount }}</strong>
      </div>
    </div>

    <div class="servers-summary-bar__actions">
      <SLButton variant="primary" @click="emit('create')">{{
        i18n.t("servers.next.summary.create")
      }}</SLButton>
      <SLButton variant="secondary" @click="emit('importExisting')">{{
        i18n.t("servers.next.summary.import")
      }}</SLButton>
      <SLButton variant="ghost" :loading="refreshing" @click="emit('refresh')">{{
        i18n.t("servers.next.summary.refresh")
      }}</SLButton>
    </div>
  </section>
</template>

<style scoped>
.servers-summary-bar {
  display: flex;
  gap: 16px;
  justify-content: space-between;
  align-items: stretch;
  padding: 18px 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  border-radius: 20px;
  background:
    linear-gradient(
      135deg,
      color-mix(in srgb, var(--sl-primary) 10%, transparent),
      transparent 45%
    ),
    color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.servers-summary-bar__metrics {
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 180px));
  gap: 12px;
}

.servers-summary-bar__metric-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
}

.servers-summary-bar__metric-card--running {
  border-color: color-mix(in srgb, var(--sl-success) 28%, transparent);
  background: color-mix(in srgb, var(--sl-success) 8%, var(--sl-surface));
}

.servers-summary-bar__metric-label {
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.servers-summary-bar__metric-value {
  color: var(--sl-text-primary);
  font-size: clamp(1.5rem, 2vw, 2rem);
  line-height: 1;
}

.servers-summary-bar__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
}

@media (max-width: 900px) {
  .servers-summary-bar {
    flex-direction: column;
  }

  .servers-summary-bar__actions {
    justify-content: flex-start;
  }
}

@media (max-width: 640px) {
  .servers-summary-bar__metrics {
    grid-template-columns: 1fr;
  }

  .servers-summary-bar__actions :deep(.sl-button) {
    flex: 1 1 calc(50% - 10px);
  }
}
</style>
