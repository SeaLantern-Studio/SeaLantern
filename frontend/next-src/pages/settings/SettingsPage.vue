<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SettingsClassicBridgeCard from "../../components/settings/SettingsClassicBridgeCard.vue";
import SettingsEntryCard from "../../components/settings/SettingsEntryCard.vue";
import SettingsSummaryBar from "../../components/settings/SettingsSummaryBar.vue";
import { useSettingsPage } from "./useSettingsPage";

const {
  bootstrapping,
  refreshing,
  pageErrorMessage,
  summaryChips,
  summaryFacts,
  entryItems,
  nextCoverageItems,
  classicCoverageItems,
  loadPage,
  performAction,
} = useSettingsPage();
</script>

<template>
  <div class="settings-page">
    <SettingsSummaryBar
      :chips="summaryChips"
      :facts="summaryFacts"
      :refreshing="refreshing"
      @refresh="loadPage(true)"
    />

    <section
      v-if="pageErrorMessage"
      class="settings-page__error-banner"
      role="alert"
      aria-live="polite"
    >
      <div class="settings-page__error-copy">
        <strong>读取设置上下文时出现问题</strong>
        <span>{{ pageErrorMessage }}</span>
      </div>

      <div class="settings-page__error-actions">
        <SLButton variant="secondary" size="sm" :loading="refreshing" @click="loadPage(true)">
          重试
        </SLButton>
      </div>
    </section>

    <section v-if="bootstrapping" class="settings-page__loading-card">
      <div class="settings-page__spinner" aria-hidden="true"></div>
      <div>
        <strong class="settings-page__loading-title">正在整理设置入口</strong>
        <p class="settings-page__loading-description">
          会先读取当前设置状态，再补充开发者入口与会话上下文说明。
        </p>
      </div>
    </section>

    <section v-else class="settings-page__entry-grid">
      <SettingsEntryCard
        v-for="entry in entryItems"
        :key="entry.id"
        :eyebrow="entry.eyebrow"
        :title="entry.title"
        :description="entry.description"
        :coverage-label="entry.coverageLabel"
        :coverage-tone="entry.coverageTone"
        :meta-items="entry.metaItems"
        :notes="entry.notes"
        :actions="entry.actions"
        @action="performAction"
      />
    </section>

    <SettingsClassicBridgeCard
      :next-items="nextCoverageItems"
      :classic-items="classicCoverageItems"
      @action="performAction"
    />
  </div>
</template>

<style scoped>
.settings-page {
  min-width: 0;
  display: grid;
  gap: 18px;
}

.settings-page__error-banner {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.22);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.settings-page__error-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.settings-page__error-copy strong {
  font-size: 0.95rem;
}

.settings-page__error-copy span {
  line-height: 1.5;
  word-break: break-word;
}

.settings-page__error-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.settings-page__loading-card {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 20px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
}

.settings-page__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: settings-page-spin 0.9s linear infinite;
}

.settings-page__loading-title {
  display: block;
  margin-bottom: 4px;
  color: var(--sl-text-primary);
}

.settings-page__loading-description {
  margin: 0;
  color: var(--sl-text-secondary);
}

.settings-page__entry-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

@keyframes settings-page-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 900px) {
  .settings-page__entry-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .settings-page__error-banner,
  .settings-page__loading-card {
    flex-direction: column;
  }
}
</style>
