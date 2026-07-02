<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import ColorThemeCard from "@components/views/paint/ColorThemeCard.vue";
import ErrorBanner from "@components/views/paint/ErrorBanner.vue";
import ImportSettingsModal from "@components/views/paint/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/paint/ResetConfirmModal.vue";
import SettingsActions from "@components/views/paint/SettingsActions.vue";
import TextCustomizationCard from "@components/views/paint/TextCustomizationCard.vue";
import ConsoleSettingsCard from "@components/views/settings/ConsoleSettingsCard.vue";
import WorkbenchFactGrid from "@next-src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPageIntro from "@next-src/components/workbench/WorkbenchPageIntro.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import { i18n } from "@language";
import { usePaintPage } from "./usePaintPage";

const {
  fields,
  appearanceOptions,
  settingsDraft,
  settings,
  personalizationBusy,
  exportPersonalizationPackage,
  importPersonalizationPackage,
  summaryFacts,
  markChanged,
  openAppearanceSettings,
  resetSettings,
  handleImport,
} = usePaintPage();
</script>

<template>
  <div class="paint-page">
    <WorkbenchPageIntro
      :eyebrow="i18n.t('common.personalize')"
      :title="i18n.t('common.personalize')"
      :description="i18n.t('settings.personalization_package_desc')"
    />

    <WorkbenchFactGrid :items="summaryFacts" />

    <ErrorBanner :message="settingsDraft.error.value" @close="settingsDraft.setError(null)" />

    <section v-if="settingsDraft.loading.value" class="paint-page__loading">
      <div class="paint-page__loading-card"></div>
      <div class="paint-page__loading-card"></div>
    </section>

    <template v-else-if="settings">
      <WorkbenchPanel
        :title="i18n.t('settings.appearance')"
        :description="i18n.t('settings.next.appearance.live_hint')"
      >
        <div class="paint-page__appearance-bridge">
          <p class="paint-page__appearance-text">
            {{ i18n.t("settings.appearance_desc") }}
          </p>
          <SLButton variant="secondary" size="sm" @click="openAppearanceSettings">
            {{ i18n.t("settings.appearance") }}
          </SLButton>
        </div>
      </WorkbenchPanel>

      <ColorThemeCard
        :color="settings.color"
        :is-theme-proxied="appearanceOptions.isThemeProxied.value"
        :theme-proxy-plugin-name="appearanceOptions.themeProxyPluginName.value"
        @update:color="settings.color = $event"
        @change="markChanged"
      />

      <TextCustomizationCard
        :app-display-name="settings.app_display_name"
        :theme="settings.theme"
        :color="settings.color"
        :text-color-overrides="settings.text_color_overrides"
        :window-effect="settings.window_effect"
        @update:app-display-name="settings.app_display_name = $event"
        @update:text-color-overrides="settings.text_color_overrides = $event"
        @change="markChanged"
      />

      <SLCard
        :title="i18n.t('settings.memory_display_precision')"
        :subtitle="i18n.t('settings.memory_display_precision_desc')"
      >
        <div class="paint-page__precision-row">
          <div class="paint-page__precision-select">
            <SLSelect
              :model-value="fields.memoryDisplayPrecision.value"
              :options="appearanceOptions.memoryDisplayPrecisionOptions.value"
              @update:model-value="
                fields.memoryDisplayPrecision.value = Number($event);
                markChanged();
              "
            />
          </div>
        </div>
      </SLCard>

      <ConsoleSettingsCard
        :console-font-size="fields.consoleFontSize.value"
        :console-font-family="fields.consoleFontFamily.value"
        :console-letter-spacing="fields.consoleLetterSpacing.value"
        :max-log-lines="fields.maxLogLines.value"
        :font-family-options="appearanceOptions.fontFamilyOptions.value"
        :fonts-loading="appearanceOptions.fontsLoading.value"
        @update:console-font-size="fields.consoleFontSize.value = $event"
        @update:console-font-family="fields.consoleFontFamily.value = $event"
        @update:console-letter-spacing="fields.consoleLetterSpacing.value = $event"
        @update:max-log-lines="fields.maxLogLines.value = $event"
        @change="markChanged"
      />

      <SettingsActions
        :busy="personalizationBusy"
        @export-package="exportPersonalizationPackage"
        @import-package="importPersonalizationPackage"
        @reset="settingsDraft.showResetConfirm.value = true"
      />
    </template>

    <ImportSettingsModal
      :visible="settingsDraft.showImportModal.value"
      @update:visible="settingsDraft.showImportModal.value = $event"
      @import="handleImport"
    />

    <ResetConfirmModal
      :visible="settingsDraft.showResetConfirm.value"
      @update:visible="settingsDraft.showResetConfirm.value = $event"
      @confirm="resetSettings"
    />
  </div>
</template>

<style scoped>
.paint-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.paint-page__appearance-bridge {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.paint-page__appearance-text {
  margin: 0;
  font-size: 0.87rem;
  line-height: 1.5;
  color: var(--sl-text-secondary);
}

.paint-page__precision-row {
  display: flex;
  justify-content: flex-end;
}

.paint-page__precision-select {
  width: min(280px, 100%);
}

.paint-page__loading {
  display: grid;
  gap: 14px;
}

.paint-page__loading-card {
  min-height: 180px;
  border-radius: 22px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 0%,
    color-mix(in srgb, var(--sl-surface) 92%, transparent) 50%,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 100%
  );
  background-size: 200% 100%;
  animation: paint-page-skeleton 1.2s ease-in-out infinite;
}

@keyframes paint-page-skeleton {
  0% {
    background-position: 100% 0;
  }

  100% {
    background-position: -100% 0;
  }
}

@media (max-width: 720px) {
  .paint-page__appearance-bridge {
    flex-direction: column;
    align-items: flex-start;
  }

  .paint-page__precision-row {
    justify-content: stretch;
  }

  .paint-page__precision-select {
    width: 100%;
  }
}
</style>
