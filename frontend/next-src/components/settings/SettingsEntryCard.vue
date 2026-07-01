<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import type {
  SettingsCoverageTone,
  SettingsEntryAction,
} from "@next-src/pages/settings/useSettingsPage";

interface Props {
  eyebrow: string;
  title: string;
  description: string;
  coverageLabel: string;
  coverageTone: SettingsCoverageTone;
  metaItems: string[];
  notes: string[];
  actions: SettingsEntryAction[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "action", actionId: string): void;
}>();
</script>

<template>
  <SLCard variant="outline" padding="lg" class="settings-entry-card">
    <div class="settings-entry-card__header">
      <div class="settings-entry-card__copy">
        <span class="settings-entry-card__eyebrow">{{ props.eyebrow }}</span>
        <h3 class="settings-entry-card__title">{{ props.title }}</h3>
        <p class="settings-entry-card__description">{{ props.description }}</p>
      </div>

      <span
        class="settings-entry-card__coverage"
        :class="`settings-entry-card__coverage--${props.coverageTone}`"
      >
        {{ props.coverageLabel }}
      </span>
    </div>

    <ul class="settings-entry-card__meta">
      <li v-for="item in props.metaItems" :key="item">{{ item }}</li>
    </ul>

    <div class="settings-entry-card__notes">
      <p v-for="note in props.notes" :key="note">{{ note }}</p>
    </div>

    <div class="settings-entry-card__actions">
      <SLButton
        v-for="action in props.actions"
        :key="action.id"
        :variant="action.variant ?? 'secondary'"
        size="sm"
        :disabled="action.disabled"
        @click="emit('action', action.id)"
      >
        {{ action.label }}
      </SLButton>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-entry-card {
  height: 100%;
  display: grid;
  gap: 16px;
}

.settings-entry-card__header {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
}

.settings-entry-card__copy {
  min-width: 0;
  display: grid;
  gap: 8px;
}

.settings-entry-card__eyebrow {
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.settings-entry-card__title,
.settings-entry-card__description,
.settings-entry-card__notes p {
  margin: 0;
}

.settings-entry-card__title {
  font-size: 1.1rem;
  color: var(--sl-text-primary);
}

.settings-entry-card__description {
  color: var(--sl-text-secondary);
  line-height: 1.65;
}

.settings-entry-card__coverage {
  flex: none;
  min-height: 30px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  font-size: 0.8rem;
  border: 1px solid transparent;
  white-space: nowrap;
}

.settings-entry-card__coverage--next {
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
  border-color: color-mix(in srgb, var(--sl-primary) 26%, transparent);
  color: var(--sl-primary);
}

.settings-entry-card__coverage--hybrid {
  background: rgba(245, 158, 11, 0.12);
  border-color: rgba(245, 158, 11, 0.28);
  color: #b45309;
}

.settings-entry-card__coverage--classic {
  background: color-mix(in srgb, var(--sl-bg-secondary) 80%, transparent);
  border-color: color-mix(in srgb, var(--sl-border) 82%, transparent);
  color: var(--sl-text-secondary);
}

.settings-entry-card__meta {
  margin: 0;
  padding-left: 18px;
  display: grid;
  gap: 8px;
  color: var(--sl-text-primary);
}

.settings-entry-card__meta li {
  line-height: 1.55;
}

.settings-entry-card__notes {
  display: grid;
  gap: 8px;
}

.settings-entry-card__notes p {
  color: var(--sl-text-secondary);
  line-height: 1.6;
}

.settings-entry-card__actions {
  margin-top: auto;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

@media (max-width: 720px) {
  .settings-entry-card__header {
    flex-direction: column;
  }
}
</style>
