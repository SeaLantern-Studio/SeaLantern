<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import type { SettingsCoverageItem } from "@next-src/pages/settings/useSettingsPage";

interface Props {
  nextItems: SettingsCoverageItem[];
  classicItems: SettingsCoverageItem[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "action", actionId: string): void;
}>();
</script>

<template>
  <SLCard variant="outline" padding="lg" class="settings-classic-bridge-card">
    <div class="settings-classic-bridge-card__header">
      <div>
        <span class="settings-classic-bridge-card__eyebrow">{{
          i18n.t("settings.next.bridge.eyebrow")
        }}</span>
        <h3 class="settings-classic-bridge-card__title">
          {{ i18n.t("settings.next.bridge.title") }}
        </h3>
      </div>

      <div class="settings-classic-bridge-card__actions">
        <SLButton variant="secondary" size="sm" @click="emit('action', 'open-paint')">
          {{ i18n.t("settings.next.bridge.open_personalization") }}
        </SLButton>
        <SLButton variant="ghost" size="sm" @click="emit('action', 'open-developer')">
          {{ i18n.t("settings.next.bridge.open_developer") }}
        </SLButton>
      </div>
    </div>

    <p class="settings-classic-bridge-card__description">
      {{ i18n.t("settings.next.bridge.description") }}
    </p>

    <div class="settings-classic-bridge-card__grid">
      <section class="settings-classic-bridge-card__column">
        <h4>{{ i18n.t("settings.next.bridge.next_section_title") }}</h4>
        <ul>
          <li v-for="item in props.nextItems" :key="item.title">
            <strong>{{ item.title }}</strong>
            <span>{{ item.description }}</span>
          </li>
        </ul>
      </section>

      <section class="settings-classic-bridge-card__column">
        <h4>{{ i18n.t("settings.next.bridge.classic_section_title") }}</h4>
        <ul>
          <li v-for="item in props.classicItems" :key="item.title">
            <strong>{{ item.title }}</strong>
            <span>{{ item.description }}</span>
          </li>
        </ul>
      </section>
    </div>
  </SLCard>
</template>

<style scoped>
.settings-classic-bridge-card {
  display: grid;
  gap: 18px;
}

.settings-classic-bridge-card__header {
  display: flex;
  gap: 16px;
  justify-content: space-between;
  align-items: flex-start;
}

.settings-classic-bridge-card__eyebrow {
  display: inline-block;
  margin-bottom: 8px;
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.settings-classic-bridge-card__title,
.settings-classic-bridge-card__description,
.settings-classic-bridge-card__column h4,
.settings-classic-bridge-card__column ul {
  margin: 0;
}

.settings-classic-bridge-card__title {
  font-size: 1.1rem;
}

.settings-classic-bridge-card__actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.settings-classic-bridge-card__description {
  color: var(--sl-text-secondary);
  line-height: 1.7;
}

.settings-classic-bridge-card__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.settings-classic-bridge-card__column {
  padding: 16px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, transparent);
}

.settings-classic-bridge-card__column h4 {
  margin-bottom: 12px;
  font-size: 0.95rem;
  color: var(--sl-text-primary);
}

.settings-classic-bridge-card__column ul {
  padding-left: 18px;
  display: grid;
  gap: 12px;
}

.settings-classic-bridge-card__column li {
  display: grid;
  gap: 4px;
}

.settings-classic-bridge-card__column strong {
  color: var(--sl-text-primary);
}

.settings-classic-bridge-card__column span {
  color: var(--sl-text-secondary);
  line-height: 1.55;
}

@media (max-width: 900px) {
  .settings-classic-bridge-card__grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .settings-classic-bridge-card__header {
    flex-direction: column;
  }

  .settings-classic-bridge-card__actions {
    justify-content: flex-start;
  }
}
</style>
