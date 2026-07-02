<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

defineProps<{
  totalCount: number;
  enabledCount: number;
  updateCount: number;
  refreshing: boolean;
  checkingUpdates: boolean;
}>();

defineEmits<{
  (e: "refresh"): void;
  (e: "check-updates"): void;
  (e: "import-local"): void;
  (e: "open-market"): void;
}>();
</script>

<template>
  <section class="plugins-summary-bar">
    <div class="plugins-summary-bar__metrics">
      <article class="plugins-summary-bar__metric-card">
        <span class="plugins-summary-bar__metric-label">{{
          i18n.t("plugins.next.summary.installed_label")
        }}</span>
        <strong class="plugins-summary-bar__metric-value">{{ totalCount }}</strong>
        <span class="plugins-summary-bar__metric-hint">{{
          i18n.t("plugins.next.summary.installed_hint")
        }}</span>
      </article>

      <article class="plugins-summary-bar__metric-card">
        <span class="plugins-summary-bar__metric-label">{{
          i18n.t("plugins.next.summary.enabled_label")
        }}</span>
        <strong class="plugins-summary-bar__metric-value">{{ enabledCount }}</strong>
        <span class="plugins-summary-bar__metric-hint">{{
          i18n.t("plugins.next.summary.enabled_hint")
        }}</span>
      </article>

      <article class="plugins-summary-bar__metric-card">
        <span class="plugins-summary-bar__metric-label">{{
          i18n.t("plugins.next.summary.updates_label")
        }}</span>
        <strong class="plugins-summary-bar__metric-value">{{ updateCount }}</strong>
        <span class="plugins-summary-bar__metric-hint">{{
          i18n.t("plugins.next.summary.updates_hint")
        }}</span>
      </article>
    </div>

    <div class="plugins-summary-bar__actions">
      <SLButton variant="secondary" size="sm" @click="$emit('import-local')">
        {{ i18n.t("settings.import") }}
      </SLButton>
      <SLButton variant="secondary" size="sm" :loading="refreshing" @click="$emit('refresh')">
        {{ i18n.t("plugins.next.summary.refresh") }}
      </SLButton>
      <SLButton
        variant="secondary"
        size="sm"
        :loading="checkingUpdates"
        @click="$emit('check-updates')"
      >
        {{ i18n.t("plugins.next.summary.check_updates") }}
      </SLButton>
      <SLButton variant="ghost" size="sm" @click="$emit('open-market')">
        {{ i18n.t("plugins.next.summary.open_market") }}
      </SLButton>
    </div>
  </section>
</template>

<style scoped>
.plugins-summary-bar {
  display: grid;
  gap: 16px;
}

.plugins-summary-bar__metrics {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.plugins-summary-bar__metric-card {
  min-width: 0;
  padding: 18px;
  display: grid;
  gap: 6px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.plugins-summary-bar__metric-label,
.plugins-summary-bar__metric-hint {
  color: var(--sl-text-secondary);
}

.plugins-summary-bar__metric-label {
  font-size: 0.8rem;
}

.plugins-summary-bar__metric-value {
  font-size: 1.8rem;
  line-height: 1;
  color: var(--sl-text-primary);
}

.plugins-summary-bar__metric-hint {
  font-size: 0.78rem;
  line-height: 1.5;
}

.plugins-summary-bar__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
}

@media (max-width: 900px) {
  .plugins-summary-bar__metrics {
    grid-template-columns: 1fr;
  }

  .plugins-summary-bar__actions {
    justify-content: flex-start;
  }
}
</style>
