<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import ErrorBanner from "@components/views/paint/ErrorBanner.vue";
import ColorThemeCard from "@components/views/paint/ColorThemeCard.vue";
import AppearanceCard from "@components/views/paint/AppearanceCard.vue";
import SettingsActions from "@components/views/paint/SettingsActions.vue";
import ImportSettingsModal from "@components/views/paint/ImportSettingsModal.vue";
import ResetConfirmModal from "@components/views/paint/ResetConfirmModal.vue";
import {
  settingsApi,
  checkAcrylicSupport,
  applyAcrylic,
  getSystemFonts,
  type AppSettings,
} from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import {
  dispatchSettingsUpdate,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const error = ref<string | null>(null);

const acrylicSupported = ref(true);
const pluginStore = usePluginStore();

const themeProxyPlugin = computed(() => {
  return pluginStore.plugins.find(
    (p) => p.state === "enabled" && pluginStore.hasCapability(p.manifest.id, "theme-provider"),
  );
});

const isThemeProxied = computed(() => !!themeProxyPlugin.value);
const themeProxyPluginName = computed(() => themeProxyPlugin.value?.manifest.name || "");

const fontSize = ref("14");
const consoleFontSize = ref("13");
const bgOpacity = ref("0.3");
const bgBlur = ref("0");
const bgBrightness = ref("1.0");

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("settings.font_family_default"), value: "" },
]);

const showImportModal = ref(false);
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);

onMounted(async () => {
  await loadSettings();
  await loadSystemFonts();
  try {
    acrylicSupported.value = await checkAcrylicSupport();
  } catch {
    acrylicSupported.value = false;
  }

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
});

onUnmounted(() => {
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
  }
});

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>) {
  const { settings: newSettings } = e.detail;
  settings.value = newSettings;
  syncLocalValues(newSettings);
}

function syncLocalValues(s: AppSettings) {
  fontSize.value = String(s.font_size);
  consoleFontSize.value = String(s.console_font_size);
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
    const s = await settingsApi.get();
    settings.value = s;
    fontSize.value = String(s.font_size);
    consoleFontSize.value = String(s.console_font_size);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    settings.value.color = s.color || "default";
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
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

function getEffectiveTheme(theme: string): "light" | "dark" {
  if (theme === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme as "light" | "dark";
}

function applyTheme(theme: string) {
  const effectiveTheme = getEffectiveTheme(theme);
  document.documentElement.setAttribute("data-theme", effectiveTheme);
  return effectiveTheme;
}

function applyFontSize(size: number) {
  document.documentElement.style.fontSize = `${size}px`;
}

function handleFontSizeChange() {
  markChanged();
  const size = parseInt(fontSize.value) || 14;
  applyFontSize(size);
}

function applyFontFamily(fontFamily: string) {
  if (fontFamily) {
    document.documentElement.style.setProperty("--sl-font-sans", fontFamily);
    document.documentElement.style.setProperty("--sl-font-display", fontFamily);
  } else {
    document.documentElement.style.removeProperty("--sl-font-sans");
    document.documentElement.style.removeProperty("--sl-font-display");
  }
}

function handleFontFamilyChange() {
  markChanged();
  if (settings.value) {
    applyFontFamily(settings.value.font_family);
  }
}

async function handleAcrylicChange(enabled: boolean) {
  markChanged();
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");

  if (!acrylicSupported.value) {
    return;
  }

  try {
    const theme = settings.value?.theme || "auto";
    const isDark = getEffectiveTheme(theme) === "dark";
    await applyAcrylic(enabled, isDark);
  } catch (e) {
    error.value = String(e);
  }
}

function handleMinimalModeChange(enabled: boolean) {
  markChanged();
  document.documentElement.setAttribute("data-minimal", enabled ? "true" : "false");
}

async function handleThemeChange() {
  markChanged();
  if (!settings.value) return;

  const effectiveTheme = applyTheme(settings.value.theme);

  if (settings.value.acrylic_enabled && acrylicSupported.value) {
    try {
      const isDark = effectiveTheme === "dark";
      await applyAcrylic(true, isDark);
    } catch {}
  }
}

async function saveSettings() {
  if (!settings.value) return;

  settings.value.console_font_size = parseInt(consoleFontSize.value) || 13;
  settings.value.background_opacity = parseFloat(bgOpacity.value) || 0.3;
  settings.value.background_blur = parseInt(bgBlur.value) || 0;
  settings.value.background_brightness = parseFloat(bgBrightness.value) || 1.0;
  settings.value.font_size = parseInt(fontSize.value) || 14;
  settings.value.color = settings.value.color || "default";

  error.value = null;
  try {
    const result = await settingsApi.saveWithDiff(settings.value);

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: settings.value.theme || "auto",
        fontSize: settings.value.font_size || 14,
      }),
    );

    if (result.changed_groups.includes("Appearance")) {
      applyTheme(settings.value.theme);
      applyFontSize(settings.value.font_size);

      if (acrylicSupported.value) {
        try {
          const isDark = getEffectiveTheme(settings.value.theme) === "dark";
          await applyAcrylic(settings.value.acrylic_enabled, isDark);
        } catch {}
      }
    }

    dispatchSettingsUpdate(result.changed_groups, result.settings);
  } catch (e) {
    error.value = String(e);
  }
}

async function resetSettings() {
  try {
    const s = await settingsApi.reset();
    settings.value = s;
    fontSize.value = String(s.font_size);
    consoleFontSize.value = String(s.console_font_size);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    showResetConfirm.value = false;
    settings.value.color = "default";

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: s.theme || "auto",
        fontSize: s.font_size || 14,
      }),
    );

    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
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
    const s = await settingsApi.importJson(json);
    settings.value = s;
    fontSize.value = String(s.font_size);
    consoleFontSize.value = String(s.console_font_size);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    showImportModal.value = false;
    applyTheme(s.theme);
    applyFontSize(s.font_size);
    applyFontFamily(s.font_family);
  } catch (e) {
    error.value = String(e);
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

      <AppearanceCard
        :theme="settings.theme"
        :font-size="fontSize"
        :font-family="settings.font_family"
        :font-family-options="fontFamilyOptions"
        :fonts-loading="fontsLoading"
        :acrylic-enabled="settings.acrylic_enabled"
        :acrylic-supported="acrylicSupported"
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
        @update:acrylic-enabled="settings.acrylic_enabled = $event"
        @update:bg-settings-expanded="bgSettingsExpanded = $event"
        @update:bg-opacity="bgOpacity = $event"
        @update:bg-blur="bgBlur = $event"
        @update:bg-brightness="bgBrightness = $event"
        @update:background-size="settings.background_size = $event"
        @update:minimal-mode="settings.minimal_mode = $event"
        @theme-change="handleThemeChange"
        @font-size-change="handleFontSizeChange"
        @font-family-change="handleFontFamilyChange"
        @acrylic-change="handleAcrylicChange"
        @minimal-mode-change="handleMinimalModeChange"
        @pick-image="pickBackgroundImage"
        @clear-image="clearBackgroundImage"
        @change="markChanged"
      />

      <SettingsActions />
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
