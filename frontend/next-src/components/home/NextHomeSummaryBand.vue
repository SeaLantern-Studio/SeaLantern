<script setup lang="ts">
import { i18n } from "@language";
import type { NextHomePageSummaryMetric } from "@next-src/pages/home/useNextHomePage";

defineProps<{
  metrics: NextHomePageSummaryMetric[];
  cardId?: string;
  section?: string;
  editable?: boolean;
}>();
</script>

<template>
  <section
    class="next-home-summary-band"
    :aria-label="i18n.t('shell.home_summary_aria_label')"
    :data-card-id="cardId"
    :data-layout-section="section"
    :data-layout-editable="editable ? 'true' : 'false'"
  >
    <article
      v-for="metric in metrics"
      :key="metric.id"
      class="next-home-summary-band__metric"
      :class="`next-home-summary-band__metric--${metric.tone ?? 'neutral'}`"
    >
      <span class="next-home-summary-band__label">{{ metric.label }}</span>
      <strong class="next-home-summary-band__value">{{ metric.value }}</strong>
      <span class="next-home-summary-band__meta">{{ metric.meta }}</span>
    </article>
  </section>
</template>

<style scoped>
.next-home-summary-band {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  border: 1px solid color-mix(in srgb, var(--sl-border) 88%, transparent);
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  overflow: hidden;
}

.next-home-summary-band__metric {
  min-width: 0;
  display: grid;
  gap: 6px;
  padding: 18px 20px;
  border-right: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.next-home-summary-band__metric:last-child {
  border-right: 0;
}

.next-home-summary-band__metric--primary {
  background: color-mix(in srgb, var(--sl-primary) 4%, var(--sl-surface));
}

.next-home-summary-band__metric--success {
  background: color-mix(in srgb, var(--sl-success) 4%, var(--sl-surface));
}

.next-home-summary-band__metric--danger {
  background: color-mix(in srgb, var(--sl-danger) 4%, var(--sl-surface));
}

.next-home-summary-band__metric--warning {
  background: color-mix(in srgb, var(--sl-warning) 4%, var(--sl-surface));
}

.next-home-summary-band__label {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.next-home-summary-band__value {
  font-size: clamp(1.4rem, 2.6vw, 2rem);
  line-height: 1;
  letter-spacing: -0.03em;
  color: var(--sl-text-primary);
}

.next-home-summary-band__meta {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

@media (max-width: 1023px) {
  .next-home-summary-band {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .next-home-summary-band__metric:nth-child(2n) {
    border-right: 0;
  }
}

@media (max-width: 639px) {
  .next-home-summary-band {
    grid-template-columns: 1fr;
  }

  .next-home-summary-band__metric {
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  }

  .next-home-summary-band__metric:last-child {
    border-bottom: 0;
  }
}
</style>
