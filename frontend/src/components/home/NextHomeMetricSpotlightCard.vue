<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import UsageGauge from "./UsageGauge.vue";
import UsageSparkline from "./UsageSparkline.vue";

type SpotlightTone = "primary" | "success" | "warning" | "danger" | "neutral";

interface SpotlightMetric {
  id: string;
  label: string;
  value: string;
  detail?: string;
  percent?: number;
  history?: number[];
  tone?: SpotlightTone;
}

const props = defineProps<{
  title: string;
  metric: SpotlightMetric;
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();

const chartTone = computed<"primary" | "success" | "warning">(() => {
  if (props.metric.tone === "success") return "success";
  if (props.metric.tone === "warning" || props.metric.tone === "danger") return "warning";
  return "primary";
});

const normalizedPercent = computed(() => {
  if (!Number.isFinite(props.metric.percent)) return null;
  return Math.min(100, Math.max(0, Math.round(props.metric.percent ?? 0)));
});

const sparklineValues = computed(() => {
  const source = props.metric.history ?? [];
  if (source.length > 0) return source;
  if (normalizedPercent.value === null) return [0, 0];
  return [normalizedPercent.value, normalizedPercent.value];
});

const isCountMetric = computed(() => normalizedPercent.value === null);
</script>

<template>
  <article
    class="next-home-metric-spotlight"
    :data-card-id="cardId"
    :data-layout-section="section"
    :data-layout-editable="editable ? 'true' : 'false'"
  >
    <header class="next-home-metric-spotlight__header">
      <span class="next-home-metric-spotlight__eyebrow">{{ title }}</span>
      <span v-if="!isCountMetric" class="next-home-metric-spotlight__percent">
        {{ normalizedPercent }}%
      </span>
    </header>

    <div class="next-home-metric-spotlight__body">
      <div v-if="!isCountMetric" class="next-home-metric-spotlight__gauge-shell">
        <UsageGauge :value="normalizedPercent ?? 0" :tone="chartTone" />
      </div>

      <div class="next-home-metric-spotlight__copy">
        <h3>{{ metric.label }}</h3>
        <strong>{{ metric.value }}</strong>
        <p v-if="metric.detail">{{ metric.detail }}</p>
      </div>
    </div>

    <div class="next-home-metric-spotlight__trend">
      <div class="next-home-metric-spotlight__trend-header">
        <span>{{ i18n.t("shell.home_metric_trend_label") }}</span>
      </div>
      <div class="next-home-metric-spotlight__sparkline-shell">
        <UsageSparkline :values="sparklineValues" :tone="chartTone" />
      </div>
    </div>
  </article>
</template>

<style scoped>
.next-home-metric-spotlight {
  min-width: 0;
  display: grid;
  gap: 16px;
  padding: 20px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 76%, white);
}

.next-home-metric-spotlight__header,
.next-home-metric-spotlight__body,
.next-home-metric-spotlight__trend-header {
  min-width: 0;
}

.next-home-metric-spotlight__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.next-home-metric-spotlight__eyebrow,
.next-home-metric-spotlight__trend-header span {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.next-home-metric-spotlight__percent {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  font-family: var(--sl-font-mono);
}

.next-home-metric-spotlight__body {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 16px;
}

.next-home-metric-spotlight__gauge-shell {
  width: 76px;
  height: 76px;
}

.next-home-metric-spotlight__copy {
  min-width: 0;
  display: grid;
  gap: 6px;
}

.next-home-metric-spotlight__copy h3,
.next-home-metric-spotlight__copy strong,
.next-home-metric-spotlight__copy p {
  margin: 0;
}

.next-home-metric-spotlight__copy h3 {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
  font-weight: 500;
}

.next-home-metric-spotlight__copy strong {
  color: var(--sl-text-primary);
  font-size: clamp(1.5rem, 2.8vw, 2rem);
  line-height: 1;
}

.next-home-metric-spotlight__copy p {
  color: var(--sl-text-secondary);
  line-height: var(--sl-line-height-relaxed);
  word-break: break-word;
}

.next-home-metric-spotlight__trend {
  display: grid;
  gap: 10px;
}

.next-home-metric-spotlight__sparkline-shell {
  height: 56px;
  border-radius: 12px;
  overflow: hidden;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
}

@media (max-width: 639px) {
  .next-home-metric-spotlight__body {
    grid-template-columns: 1fr;
  }

  .next-home-metric-spotlight__gauge-shell {
    justify-self: start;
  }
}
</style>
