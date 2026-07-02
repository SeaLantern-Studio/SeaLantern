<script setup lang="ts">
import { computed } from "vue";
import { Gauge, Menu } from "@lucide/vue";
import { i18n } from "@language";
import UsageGauge from "@src/components/views/home/UsageGauge.vue";
import UsageSparkline from "@src/components/views/home/UsageSparkline.vue";

type SystemOverviewTone = "primary" | "success" | "warning";
type SystemOverviewMode = "gauge" | "detail";

interface SystemOverviewMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
  percent: number;
  history: number[];
  tone: SystemOverviewTone;
}

const props = defineProps<{
  title: string;
  metrics: SystemOverviewMetric[];
  viewMode: SystemOverviewMode;
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{
  toggleView: [];
}>();

const toggleTitle = computed(() =>
  props.viewMode === "gauge"
    ? i18n.t("shell.home_system_toggle_to_detail")
    : i18n.t("shell.home_system_toggle_to_gauge"),
);
</script>

<template>
  <article
    class="next-home-system-overview surface-panel"
    :data-card-id="cardId"
    :data-layout-section="section"
    :data-layout-editable="editable ? 'true' : 'false'"
  >
    <header class="next-home-system-overview__header">
      <h3>{{ title }}</h3>
      <button
        class="next-home-system-overview__toggle"
        type="button"
        :title="toggleTitle"
        @click="emit('toggleView')"
      >
        <Menu v-if="viewMode === 'gauge'" :size="14" />
        <Gauge v-else :size="14" />
      </button>
    </header>

    <div v-if="viewMode === 'gauge'" class="next-home-system-overview__gauge-grid">
      <div v-for="metric in metrics" :key="metric.id" class="next-home-system-overview__gauge-item">
        <div class="next-home-system-overview__gauge-shell">
          <UsageGauge
            class="next-home-system-overview__gauge-chart"
            :value="metric.percent"
            :tone="metric.tone"
          />
        </div>
        <div class="next-home-system-overview__metric-copy">
          <span>{{ metric.label }}</span>
          <strong>{{ metric.value }}</strong>
          <small>{{ metric.detail }}</small>
        </div>
      </div>
    </div>

    <div v-else class="next-home-system-overview__detail-list">
      <div
        v-for="metric in metrics"
        :key="metric.id"
        class="next-home-system-overview__detail-item"
      >
        <div class="next-home-system-overview__detail-header">
          <span>{{ metric.label }}</span>
          <strong>{{ metric.value }}</strong>
        </div>
        <div class="next-home-system-overview__sparkline-wrap">
          <UsageSparkline
            class="next-home-system-overview__sparkline"
            :values="metric.history"
            :tone="metric.tone"
          />
        </div>
        <small>{{ metric.detail }}</small>
      </div>
    </div>
  </article>
</template>

<style scoped>
.surface-panel {
  min-width: 0;
  display: grid;
  gap: 16px;
  padding: 20px;
  align-self: start;
  height: 100%;
  max-height: 100%;
  overflow: hidden;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, white);
}

.next-home-system-overview__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.next-home-system-overview__header h3 {
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 1rem;
  font-weight: 600;
}

.next-home-system-overview__toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  border-radius: 10px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.next-home-system-overview__gauge-grid,
.next-home-system-overview__detail-list {
  display: grid;
  gap: 12px;
  align-content: start;
}

.next-home-system-overview__gauge-item,
.next-home-system-overview__detail-item {
  min-width: 0;
  display: grid;
  gap: 10px;
  padding: 14px 16px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.next-home-system-overview__gauge-item {
  grid-template-columns: 68px minmax(0, 1fr);
  align-items: center;
}

.next-home-system-overview__gauge-shell {
  width: 68px;
  height: 68px;
}

.next-home-system-overview__gauge-chart {
  width: 100%;
  height: 100%;
}

.next-home-system-overview__metric-copy,
.next-home-system-overview__detail-item {
  min-width: 0;
}

.next-home-system-overview__metric-copy {
  display: grid;
  gap: 4px;
}

.next-home-system-overview__metric-copy span,
.next-home-system-overview__detail-item span,
.next-home-system-overview__metric-copy small,
.next-home-system-overview__detail-item small {
  color: var(--sl-text-secondary);
}

.next-home-system-overview__metric-copy strong,
.next-home-system-overview__detail-item strong {
  color: var(--sl-text-primary);
  font-size: 1.1rem;
}

.next-home-system-overview__detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.next-home-system-overview__sparkline-wrap {
  height: 42px;
  border-radius: 10px;
  overflow: hidden;
  background: color-mix(in srgb, var(--sl-bg-secondary) 78%, transparent);
}

.next-home-system-overview__sparkline {
  width: 100%;
  height: 100%;
}

@media (max-width: 639px) {
  .surface-panel {
    max-height: none;
    overflow: hidden;
  }

  .next-home-system-overview__gauge-item {
    grid-template-columns: 1fr;
  }

  .next-home-system-overview__gauge-shell {
    width: 72px;
    height: 72px;
    justify-self: start;
  }
}
</style>
