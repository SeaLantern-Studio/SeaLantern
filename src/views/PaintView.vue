<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import ErrorBanner from "@components/views/paint/ErrorBanner.vue";
import ColorThemeCard from "@components/views/paint/ColorThemeCard.vue";
import AppearanceCard from "@components/views/paint/AppearanceCard.vue";
import TextCustomizationCard from "@components/views/paint/TextCustomizationCard.vue";
import ConsoleSettingsCard from "@components/views/settings/ConsoleSettingsCard.vue";
import SettingsActions from "@components/views/paint/SettingsActions.vue";
import ImportSettingsModal from "@components/views/paint/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/paint/ResetConfirmModal.vue";
import { settingsApi, getSystemFonts, type AppSettings, type WindowEffect } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useToast } from "@composables/useToast";
import { usePluginStore } from "@stores/pluginStore";
import { useSettingsStore } from "@stores/settingsStore";
import { isMacOSPlatform, isWindowsPlatform } from "@utils/platform";

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const error = ref<string | null>(null);

const pluginStore = usePluginStore();
const settingsStore = useSettingsStore();
const toast = useToast();

const themeProxyPlugin = computed(() => {
  return pluginStore.plugins.find(
    (p) => p.state === "enabled" && pluginStore.hasCapability(p.manifest.id, "theme-provider"),
  );
});

const isThemeProxied = computed(() => !!themeProxyPlugin.value);
const themeProxyPluginName = computed(() => themeProxyPlugin.value?.manifest.name || "");

const fontSize = ref("14");
const consoleFontSize = ref("13");
const consoleFontFamily = ref("");
const consoleLetterSpacing = ref("0");
const maxLogLines = ref("5000");
const bgOpacity = ref("0.3");
const bgBlur = ref("0");
const bgBrightness = ref("1.0");
const isMacOS = isMacOSPlatform();
const isWindows = isWindowsPlatform();

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("settings.font_family_default"), value: "" },
]);

const windowEffectOptions = computed<{ label: string; value: WindowEffect }[]>(() => {
  if (isMacOS) {
    return [
      { label: i18n.t("settings.window_effect_options.off"), value: "off" },
      { label: i18n.t("settings.window_effect_options.vibrancy"), value: "vibrancy" },
    ];
  }

  if (isWindows) {
    return [
      { label: i18n.t("settings.window_effect_options.off"), value: "off" },
      { label: i18n.t("settings.window_effect_options.auto"), value: "auto" },
      { label: i18n.t("settings.window_effect_options.mica"), value: "mica" },
      { label: i18n.t("settings.window_effect_options.acrylic"), value: "acrylic" },
      { label: i18n.t("settings.window_effect_options.blur"), value: "blur" },
    ];
  }

  return [{ label: i18n.t("settings.window_effect_options.off"), value: "off" }];
});

const windowEffectSummary = computed(() => {
  const effect = settings.value?.window_effect || "off";
  const key = `settings.window_effect_help.${effect}`;
  return i18n.t(key);
});

const showImportModal = ref(false);
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
const personalizationBusy = ref(false);

onMounted(async () => {
  await loadSettings();
  await loadSystemFonts();
});

onUnmounted(() => {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
});

function syncLocalValues(s: AppSettings) {
  fontSize.value = String(s.font_size);
  consoleFontSize.value = String(s.console_font_size);
  consoleFontFamily.value = s.console_font_family || "";
  consoleLetterSpacing.value = String(s.console_letter_spacing ?? 0);
  maxLogLines.value = String(s.max_log_lines);
  bgOpacity.value = String(s.background_opacity);
  bgBlur.value = String(s.background_blur);
  bgBrightness.value = String(s.background_brightness);
}

async function loadSystemFonts() {
  fontsLoading.value = true;
  try {
    const fonts = await getSystemFonts();
    fontFamilyOptions.value = [
      { label: i18n.t("settings.font_family_default"), value: "" },
      ...fonts.map((font) => ({ label: font, value: `'${font}'` })),
    ];
  } catch (e) {
    console.error("Failed to load system fonts:", e);
  } finally {
    fontsLoading.value = false;
  }
}

async function loadSettings() {
  loading.value = true;
  error.value = null;
  try {
    await settingsStore.ensureLoaded();
    const nextSettings = settingsStore.cloneSettings();
    settings.value = nextSettings;
    settings.value.color = nextSettings.color || "default";
    syncLocalValues(nextSettings);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function markChanged() {
  debouncedSave();
}

let saveTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }
  saveTimeout = setTimeout(() => {
    saveSettings();
    saveTimeout = null;
  }, 500);
}

function handleFontSizeChange() {
  markChanged();
}

function handleFontFamilyChange() {
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

async function saveSettings() {
  if (!settings.value) return;

  settings.value.console_font_size = parseInt(consoleFontSize.value) || 13;
  settings.value.console_font_family = consoleFontFamily.value;
  settings.value.console_letter_spacing = parseInt(consoleLetterSpacing.value) || 0;
  settings.value.max_log_lines = parseInt(maxLogLines.value) || 5000;
  settings.value.background_opacity = parseFloat(bgOpacity.value) || 0.3;
  settings.value.background_blur = parseInt(bgBlur.value) || 0;
  settings.value.background_brightness = parseFloat(bgBrightness.value) || 1.0;
  settings.value.font_size = parseInt(fontSize.value) || 14;
  settings.value.color = settings.value.color || "default";

  error.value = null;
  try {
    await settingsStore.saveSettingsWithDiff(settings.value);
    settings.value = settingsStore.cloneSettings();
    syncLocalValues(settings.value);
  } catch (e) {
    error.value = String(e);
  }
}

async function resetSettings() {
  try {
    const s = await settingsStore.resetSettings(["Appearance", "Console"]);
    settings.value = settingsStore.cloneSettings(s);
    syncLocalValues(settings.value);
    showResetConfirm.value = false;
    settings.value.color = "default";
  } catch (e) {
    error.value = String(e);
  }
}

async function handleImport(json: string) {
  if (!json.trim()) {
    error.value = i18n.t("settings.no_json");
    return;
  }
  try {
    const s = await settingsStore.importSettingsJson(json, ["Appearance", "Console"]);
    settings.value = settingsStore.cloneSettings(s);
    syncLocalValues(settings.value);
    showImportModal.value = false;
  } catch (e) {
    error.value = String(e);
  }
}

async function exportPersonalizationPackage() {
  personalizationBusy.value = true;
  error.value = null;
  try {
    const suggestedName = await settingsApi.getPersonalizationPackageSuggestedName();
    const filePath = await systemApi.pickPersonalizationExportFile(suggestedName);
    if (!filePath) {
      return;
    }

    await settingsApi.exportPersonalizationPackage(filePath);
    toast.success(i18n.t("settings.personalization_export_success"));
  } catch (e) {
    error.value = String(e);
    toast.error(String(e));
  } finally {
    personalizationBusy.value = false;
  }
}

async function importPersonalizationPackage() {
  personalizationBusy.value = true;
  error.value = null;
  try {
    const filePath = await systemApi.pickPersonalizationImportFile();
    if (!filePath) {
      return;
    }

    const result = await settingsApi.importPersonalizationPackage(filePath);
    settingsStore.replaceSettings(result.settings, result.changed_groups);
    settings.value = settingsStore.cloneSettings(result.settings);
    syncLocalValues(result.settings);
    await pluginStore.injectAllPluginCss();

    if (result.skipped_plugins.length > 0) {
      toast.warning(i18n.t("settings.personalization_import_partial"));
    } else {
      toast.success(i18n.t("settings.personalization_import_success"));
    }
  } catch (e) {
    error.value = String(e);
    toast.error(String(e));
  } finally {
    personalizationBusy.value = false;
  }
}

async function pickBackgroundImage() {
  try {
    const result = await systemApi.pickImageFile();
    console.log("Selected image:", result);
    if (result && settings.value) {
      settings.value.background_image = result;
      markChanged();
    }
  } catch (e) {
    console.error("Pick image error:", e);
    error.value = String(e);
  }
}

function clearBackgroundImage() {
  if (settings.value) {
    settings.value.background_image = "";
    markChanged();
  }
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
        :package-available="true"
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
