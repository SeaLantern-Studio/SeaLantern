<script setup lang="ts">
import { computed, ref } from "vue";
import ErrorBanner from "@components/views/paint/ErrorBanner.vue";
import ColorThemeCard from "@components/views/paint/ColorThemeCard.vue";
import AppearanceCard from "@components/views/paint/AppearanceCard.vue";
import TextCustomizationCard from "@components/views/paint/TextCustomizationCard.vue";
import ConsoleSettingsCard from "@components/views/settings/ConsoleSettingsCard.vue";
import SettingsActions from "@components/views/paint/SettingsActions.vue";
import ImportSettingsModal from "@components/views/paint/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/paint/ResetConfirmModal.vue";
import type { WindowEffect } from "@api/settings";
import { i18n } from "@language";
import { useSettingsPageDraft } from "@composables/useSettingsPageDraft";
import { usePaintAppearanceOptions } from "@components/views/paint/usePaintAppearanceOptions";
import { usePaintPersonalizationActions } from "@components/views/paint/usePaintPersonalizationActions";
import { usePaintSettingsFields } from "@components/views/paint/usePaintSettingsFields";

const bgSettingsExpanded = ref(false);
const fields = usePaintSettingsFields();
const appearanceOptions = usePaintAppearanceOptions();
const {
  fontSize,
  memoryDisplayPrecision,
  consoleFontSize,
  consoleFontFamily,
  consoleLetterSpacing,
  maxLogLines,
  bgOpacity,
  bgBlur,
  bgBrightness,
} = fields;
const {
  fontsLoading,
  fontFamilyOptions,
  isThemeProxied,
  themeProxyPluginName,
  windowEffectOptions,
  memoryDisplayPrecisionOptions,
} = appearanceOptions;

const windowEffectSummary = computed(() => {
  const effect = settings.value?.window_effect || "off";
  const key = `settings.window_effect_help.${effect}`;
  return i18n.t(key);
});

const settingsDraft = useSettingsPageDraft({
  changedGroups: ["Appearance", "Console"],
  syncLocalValues: fields.syncLocalValues,
  prepareForSave: fields.prepareForSave,
  emptyImportMessage: () => i18n.t("settings.no_json"),
});

const settings = settingsDraft.settings;
const loading = settingsDraft.loading;
const error = settingsDraft.error;
const showImportModal = settingsDraft.showImportModal;
const showResetConfirm = settingsDraft.showResetConfirm;

const personalizationActions = usePaintPersonalizationActions({
  settings,
  settingsDraft,
  markChanged: settingsDraft.markChanged,
});
const {
  personalizationBusy,
  exportPersonalizationPackage,
  importPersonalizationPackage,
  pickBackgroundImage,
  clearBackgroundImage,
} = personalizationActions;

function markChanged() {
  settingsDraft.markChanged();
}

function handleFontSizeChange() {
  markChanged();
}

function handleFontFamilyChange() {
  markChanged();
}

function handleMemoryDisplayPrecisionChange() {
  markChanged();
}

function handleWindowEffectChange(_effect: WindowEffect) {
  markChanged();
}

function handleMinimalModeChange(_enabled: boolean) {
  markChanged();
}

function handleThemeChange() {
  markChanged();
}

async function resetSettings() {
  await settingsDraft.resetSettings();
}

async function handleImport(json: string) {
  await settingsDraft.importSettings(json);
}
</script>

<template>
  <div class="settings-view animate-fade-in-up">
    <ErrorBanner :message="error" @close="error = null" />

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("settings.loading") }}</span>
    </div>

    <template v-else-if="settings">
      <ColorThemeCard
        :color="settings.color"
        :is-theme-proxied="isThemeProxied"
        :theme-proxy-plugin-name="themeProxyPluginName"
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

      <AppearanceCard
        :theme="settings.theme"
        :font-size="fontSize"
        :font-family="settings.font_family"
        :memory-display-precision="memoryDisplayPrecision"
        :memory-display-precision-options="memoryDisplayPrecisionOptions"
        :font-family-options="fontFamilyOptions"
        :fonts-loading="fontsLoading"
        :window-effect="settings.window_effect"
        :window-effect-options="windowEffectOptions"
        :window-effect-summary="windowEffectSummary"
        :is-theme-proxied="isThemeProxied"
        :theme-proxy-plugin-name="themeProxyPluginName"
        :background-image="settings.background_image"
        :bg-opacity="bgOpacity"
        :bg-blur="bgBlur"
        :bg-brightness="bgBrightness"
        :background-size="settings.background_size"
        :bg-settings-expanded="bgSettingsExpanded"
        :minimal-mode="settings.minimal_mode"
        @update:theme="settings.theme = $event"
        @update:font-size="fontSize = $event"
        @update:font-family="settings.font_family = $event"
        @update:memory-display-precision="memoryDisplayPrecision = $event"
        @update:window-effect="settings.window_effect = $event"
        @update:bg-settings-expanded="bgSettingsExpanded = $event"
        @update:bg-opacity="bgOpacity = $event"
        @update:bg-blur="bgBlur = $event"
        @update:bg-brightness="bgBrightness = $event"
        @update:background-size="settings.background_size = $event"
        @update:minimal-mode="settings.minimal_mode = $event"
        @theme-change="handleThemeChange"
        @font-size-change="handleFontSizeChange"
        @font-family-change="handleFontFamilyChange"
        @memory-display-precision-change="handleMemoryDisplayPrecisionChange"
        @window-effect-change="handleWindowEffectChange"
        @minimal-mode-change="handleMinimalModeChange"
        @pick-image="pickBackgroundImage"
        @clear-image="clearBackgroundImage"
        @change="markChanged"
      />

      <ConsoleSettingsCard
        v-model:consoleFontSize="consoleFontSize"
        v-model:consoleFontFamily="consoleFontFamily"
        v-model:consoleLetterSpacing="consoleLetterSpacing"
        v-model:maxLogLines="maxLogLines"
        :fontFamilyOptions="fontFamilyOptions"
        :fontsLoading="fontsLoading"
        @change="markChanged"
      />

      <SettingsActions
        :busy="personalizationBusy"
        @export-package="exportPersonalizationPackage"
        @import-package="importPersonalizationPackage"
        @reset="showResetConfirm = true"
      />
    </template>

    <ImportSettingsModal v-model:visible="showImportModal" @import="handleImport" />

    <ResetConfirmModal v-model:visible="showResetConfirm" @confirm="resetSettings" />
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 860px;
  margin: 0 auto;
  padding-bottom: var(--sl-space-2xl);
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: sl-spin 0.8s linear infinite;
}

@keyframes sl-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
