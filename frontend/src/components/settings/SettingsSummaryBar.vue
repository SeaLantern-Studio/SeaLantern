<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { SettingsSummaryChip, SettingsSummaryFact } from "@src/pages/settings/useSettingsPage";

interface Props {
  chips: SettingsSummaryChip[];
  facts: SettingsSummaryFact[];
  refreshing?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  refreshing: false,
});

const emit = defineEmits<{
  (e: "refresh"): void;
}>();
</script>

<template>
  <SLCard variant="outline" padding="lg" class="settings-summary-bar">
    <div class="settings-summary-bar__header">
      <div class="settings-summary-bar__copy">
        <span class="settings-summary-bar__eyebrow">{{
          i18n.t("settings.next.summary.eyebrow")
        }}</span>
        <h2 class="settings-summary-bar__title">{{ i18n.t("settings.next.summary.title") }}</h2>
        <p class="settings-summary-bar__description">
          {{ i18n.t("settings.next.summary.description") }}
        </p>
      </div>

      <div class="settings-summary-bar__actions">
        <SLButton
          variant="secondary"
          size="sm"
          :loading="props.refreshing"
          @click="emit('refresh')"
        >
          {{ i18n.t("settings.next.summary.refresh") }}
        </SLButton>
      </div>
    </div>

    <div
      class="settings-summary-bar__chips"
      :aria-label="i18n.t('settings.next.summary.chips_aria_label')"
    >
      <span
        v-for="chip in props.chips"
        :key="chip.label"
        class="settings-summary-bar__chip"
        :class="`settings-summary-bar__chip--${chip.tone}`"
      >
        {{ chip.label }}
      </span>
    </div>

    <dl class="settings-summary-bar__facts">
      <div v-for="fact in props.facts" :key="fact.label" class="settings-summary-bar__fact">
        <dt>{{ fact.label }}</dt>
        <dd>{{ fact.value }}</dd>
      </div>
    </dl>
  </SLCard>
</template>

<style scoped>
.settings-summary-bar {
  display: grid;
  gap: 18px;
}

.settings-summary-bar__header {
  display: flex;
  gap: 16px;
  justify-content: space-between;
  align-items: flex-start;
}

.settings-summary-bar__copy {
  min-width: 0;
  display: grid;
  gap: 8px;
}

.settings-summary-bar__eyebrow {
  font-size: 0.76rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.settings-summary-bar__title,
.settings-summary-bar__description {
  margin: 0;
}

.settings-summary-bar__title {
  font-size: clamp(1.5rem, 2vw, 2rem);
  line-height: 1.05;
}

.settings-summary-bar__description {
  max-width: 64ch;
  color: var(--sl-text-secondary);
  line-height: 1.65;
}

.settings-summary-bar__actions {
  display: flex;
  justify-content: flex-end;
}

.settings-summary-bar__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.settings-summary-bar__chip {
  min-height: 34px;
  padding: 0 12px;
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 0.84rem;
}

.settings-summary-bar__chip--primary {
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
  border-color: color-mix(in srgb, var(--sl-primary) 26%, transparent);
  color: var(--sl-primary);
}

.settings-summary-bar__chip--neutral {
  background: color-mix(in srgb, var(--sl-bg-secondary) 80%, transparent);
  border-color: color-mix(in srgb, var(--sl-border) 82%, transparent);
  color: var(--sl-text-secondary);
}

.settings-summary-bar__chip--warning {
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.28);
  color: #b45309;
}

.settings-summary-bar__facts {
  margin: 0;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}

.settings-summary-bar__fact {
  min-width: 0;
  padding: 14px 16px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 78%, transparent);
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
}

.settings-summary-bar__fact dt,
.settings-summary-bar__fact dd {
  margin: 0;
}

.settings-summary-bar__fact dt {
  margin-bottom: 6px;
  font-size: 0.8rem;
  color: var(--sl-text-tertiary);
}

.settings-summary-bar__fact dd {
  color: var(--sl-text-primary);
  font-weight: 600;
  line-height: 1.45;
  word-break: break-word;
}

@media (max-width: 960px) {
  .settings-summary-bar__facts {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 720px) {
  .settings-summary-bar__header {
    flex-direction: column;
  }

  .settings-summary-bar__actions {
    justify-content: flex-start;
  }

  .settings-summary-bar__facts {
    grid-template-columns: 1fr;
  }
}
</style>
