<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import AppearanceSection from "@src/components/settings/appearance/AppearanceSection.vue";
import ColorThemeCard from "@src/components/paint/ColorThemeCard.vue";
import ErrorBanner from "@src/components/paint/ErrorBanner.vue";
import SettingsActions from "@src/components/paint/SettingsActions.vue";
import TextCustomizationCard from "@src/components/paint/TextCustomizationCard.vue";
import ConsoleSettingsCard from "@src/components/settings/ConsoleSettingsCard.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchSectionHeader from "@src/components/workbench/WorkbenchSectionHeader.vue";
import WorkbenchSplitView from "@src/components/workbench/WorkbenchSplitView.vue";
import { i18n } from "@language";
import { usePaintPage } from "./usePaintPage";

const {
  activeSectionId,
  currentSection,
  fields,
  appearanceOptions,
  settingsDraft,
  settings,
  exportBusy,
  importBusy,
  exportPersonalizationPackage,
  importPersonalizationPackage,
  sectionItems,
  summaryFacts,
  markChanged,
  selectSection,
} = usePaintPage();
</script>

<template>
  <div class="paint-page">
    <WorkbenchFactGrid :items="summaryFacts" />

    <ErrorBanner :message="settingsDraft.error.value" @close="settingsDraft.setError(null)" />

    <section v-if="settingsDraft.loading.value" class="paint-page__loading">
      <div class="paint-page__loading-card"></div>
      <div class="paint-page__loading-card"></div>
    </section>

    <WorkbenchSplitView
      v-else-if="settings"
      :items="sectionItems"
      :active-id="activeSectionId"
      :aria-label="i18n.t('settings.paint.nav_aria_label')"
      :ariaLabel="i18n.t('settings.paint.nav_aria_label')"
      @select="selectSection"
    >
      <template #content-header>
        <WorkbenchSectionHeader :title="currentSection.label" :description="currentSection.description" />
      </template>

      <template v-if="activeSectionId === 'appearance-theme'">
        <AppearanceSection :show-personalize-bridge="false" />

        <ColorThemeCard
          :color="settings.color"
          :is-theme-provider-active="appearanceOptions.isThemeProviderActive.value"
          :theme-provider-notice="appearanceOptions.themeProviderNotice.value"
          @update:color="settings.color = $event"
          @change="markChanged"
        />
      </template>

      <template v-else-if="activeSectionId === 'text-title'">
        <TextCustomizationCard
          :app-display-name="settings.app_display_name"
          :theme="settings.theme"
          :color="settings.color"
          :text-color-overrides="settings.text_color_overrides"
          :window-effect="settings.window_effect"
          :is-theme-provider-active="appearanceOptions.isThemeProviderActive.value"
          :theme-provider-notice="appearanceOptions.themeProviderNotice.value"
          @update:app-display-name="settings.app_display_name = $event"
          @update:text-color-overrides="settings.text_color_overrides = $event"
          @change="markChanged"
        />
      </template>

      <template v-else>
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
          :import-busy="importBusy"
          :export-busy="exportBusy"
          @export-package="exportPersonalizationPackage"
          @import-package="importPersonalizationPackage"
        />
      </template>
    </WorkbenchSplitView>
  </div>
</template>

<style scoped>
.paint-page {
  min-width: 0;
  display: grid;
  gap: 16px;
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
  .paint-page__precision-row {
    justify-content: stretch;
  }

  .paint-page__precision-select {
    width: 100%;
  }
}
</style>
