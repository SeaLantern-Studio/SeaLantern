import { defineStore } from "pinia";
import { ref, computed, toRaw } from "vue";
import {
  settingsApi,
  type AppSettings,
  type DataDirChangeResult,
  type DataDirStatus,
  type ImportPersonalizationResult,
  type PluginDirChangeResult,
  type PluginDirStatus,
  type PartialSettings,
  type SettingsGroup,
  type WindowEffect,
} from "@api/settings";
import { convertFileSrc } from "@tauri-apps/api/core";
import { isWindowsPlatform } from "@utils/platform";
import {
  applyColors,
  applyDeveloperMode,
  applyFontFamily,
  applyFontSize,
  applyMinimalMode,
  applyTheme,
  applyWindowTitle,
  getEffectiveTheme as resolveEffectiveTheme,
  isThemeProviderActive,
} from "@utils/theme";

const THEME_CACHE_KEY = "sl_theme_cache";
const WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED = true;

function getThemeCache(): { theme: string; fontSize: number } | null {
  try {
    const cached = localStorage.getItem(THEME_CACHE_KEY);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch (e) {}
  return null;
}

function saveThemeCache(theme: string, fontSize: number): void {
  try {
    localStorage.setItem(THEME_CACHE_KEY, JSON.stringify({ theme, fontSize }));
  } catch (e) {}
}

function cloneSettingsSnapshot(source: AppSettings): AppSettings {
  const plainSource = toRaw(source) as AppSettings;

  if (typeof structuredClone === "function") {
    try {
      return structuredClone(plainSource);
    } catch (error) {
      console.warn("Failed to structuredClone settings, falling back to JSON clone.", error);
    }
  }
  return JSON.parse(JSON.stringify(plainSource)) as AppSettings;
}

async function exportSettingsJson(): Promise<string> {
  return settingsApi.exportJson();
}

async function getPersonalizationPackageSuggestedName(): Promise<string> {
  return settingsApi.getPersonalizationPackageSuggestedName();
}

async function exportPersonalizationPackage(path: string): Promise<void> {
  return settingsApi.exportPersonalizationPackage(path);
}

export function getInitialTheme(): string {
  const cache = getThemeCache();
  if (cache && cache.theme) {
    return cache.theme;
  }
  return "auto";
}

export function getInitialFontSize(): number {
  const cache = getThemeCache();
  if (cache && cache.fontSize) {
    return cache.fontSize;
  }
  return 14;
}

const defaultSettings: AppSettings = {
  close_servers_on_exit: true,
  close_servers_on_update: true,
  auto_accept_eula: false,
  default_max_memory: 4096,
  default_min_memory: 1024,
  default_port: 25565,
  default_java_path: "",
  default_jvm_args: [],
  default_cpu_policy: {
    mode: "off",
    count: null,
    explicit_set: null,
    sync_active_processor_count: true,
  },
  default_jvm_preset: {
    preset: "none",
  },
  console_font_size: 12,
  console_font_family: "",
  console_letter_spacing: 0,
  max_log_lines: 1000,
  cached_java_list: [],
  background_image: "",
  background_opacity: 0.3,
  background_blur: 0,
  background_brightness: 1.0,
  background_size: "cover",
  window_effect: "off",
  acrylic_enabled: false,
  theme: "auto",
  font_size: 14,
  font_family: "",
  memory_display_precision: 2,
  color: "default",
  text_color_overrides: {
    title: "",
    text: "",
    description: "",
  },
  app_display_name: "",
  language: "zh-CN",
  developer_mode: false,
  close_action: "ask",
  last_run_path: "",
  minimal_mode: false,
  agreed_to_terms: false,
};

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(defaultSettings);
  const dataDirStatus = ref<DataDirStatus | null>(null);
  const pluginDirStatus = ref<PluginDirStatus | null>(null);
  const isLoaded = ref(false);
  const isLoading = ref(false);
  const loadError = ref<string | null>(null);
  const isWindows = isWindowsPlatform();

  let loadPromise: Promise<void> | null = null;
  let appearanceApplyQueue: Promise<void> = Promise.resolve();
  let lastNativeEffectKey: string | null = null;

  const theme = computed(() => settings.value.theme || "auto");
  const fontSize = computed(() => settings.value.font_size || 14);
  const windowEffect = computed<WindowEffect>(
    () => (settings.value.window_effect || "off") as WindowEffect,
  );
  const colorScheme = computed(() => settings.value.color || "default");
  const minimalMode = computed(() => settings.value.minimal_mode || false);
  const backgroundImage = computed(() =>
    settings.value.background_image ? convertFileSrc(settings.value.background_image) : "",
  );
  const backgroundOpacity = computed(() => settings.value.background_opacity);
  const backgroundBlur = computed(() => settings.value.background_blur);
  const backgroundBrightness = computed(() => settings.value.background_brightness);
  const backgroundSize = computed(() => settings.value.background_size);

  function cloneSettings(source: AppSettings = settings.value): AppSettings {
    return cloneSettingsSnapshot(source);
  }

  function syncSettings(nextSettings: AppSettings): void {
    settings.value = nextSettings;
    isLoaded.value = true;
    saveThemeCache(nextSettings.theme || "auto", nextSettings.font_size || 14);
  }

  function replaceSettings(
    nextSettings: AppSettings,
    _changedGroups: SettingsGroup[] = [],
  ): AppSettings {
    syncSettings(nextSettings);
    return nextSettings;
  }

  function applyWindowEffectAttributes(nextSettings: AppSettings): void {
    const effect = (nextSettings.window_effect || "off") as WindowEffect;
    const nativeWindowEffectEnabled = !(isWindows && WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED);
    const hasBackgroundImage = Boolean(nextSettings.background_image);
    const enabled = nativeWindowEffectEnabled
      ? effect !== "off" || !hasBackgroundImage
      : effect !== "off" && hasBackgroundImage;
    document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");
    document.documentElement.setAttribute("data-window-effect", effect);
  }

  async function applyClientSettings(nextSettings: AppSettings = settings.value): Promise<void> {
    applyTheme(nextSettings.theme || "auto");
    applyFontSize(nextSettings.font_size || 14);
    applyFontFamily(nextSettings.font_family || "");
    applyMinimalMode(nextSettings.minimal_mode || false);
    applyDeveloperMode(nextSettings.developer_mode || false);
    await applyWindowTitle(nextSettings);

    const effect = (nextSettings.window_effect || "off") as WindowEffect;
    const dark = resolveEffectiveTheme(nextSettings.theme || "auto") === "dark";

    applyWindowEffectAttributes(nextSettings);
    if (isWindows) {
      const nativeEffectKey = `${effect}:${dark}`;
      if (lastNativeEffectKey !== nativeEffectKey) {
        lastNativeEffectKey = nativeEffectKey;
        await settingsApi.applyWindowEffect(effect, dark);
      }
    }

    if (!isThemeProviderActive()) {
      applyColors(nextSettings);
    }
  }

  function queueClientSettingsApply(nextSettings: AppSettings = settings.value): Promise<void> {
    const snapshot = cloneSettings(nextSettings);
    appearanceApplyQueue = appearanceApplyQueue.then(
      () => applyClientSettings(snapshot),
      () => applyClientSettings(snapshot),
    );
    return appearanceApplyQueue;
  }

  async function ensureLoaded(): Promise<void> {
    if (isLoaded.value) {
      return;
    }
    await loadSettings();
  }

  async function loadSettings(forceReload = false): Promise<void> {
    if (isLoaded.value && !forceReload) {
      return;
    }

    if (loadPromise) {
      return loadPromise;
    }

    isLoading.value = true;
    loadError.value = null;

    loadPromise = (async () => {
      try {
        const loadedSettings = await settingsApi.get();
        syncSettings(loadedSettings);
        dataDirStatus.value = await settingsApi.getDataDirStatus();
        pluginDirStatus.value = await settingsApi.getPluginDirStatus();
      } catch (e) {
        console.error("Failed to load settings:", e);
        loadError.value = e instanceof Error ? e.message : String(e);
        syncSettings(cloneSettings(defaultSettings));
      } finally {
        isLoading.value = false;
        loadPromise = null;
      }
    })();

    return loadPromise;
  }

  async function refreshDataDirStatus(): Promise<DataDirStatus> {
    const status = await settingsApi.getDataDirStatus();
    dataDirStatus.value = status;
    return status;
  }

  async function refreshPluginDirStatus(): Promise<PluginDirStatus> {
    const status = await settingsApi.getPluginDirStatus();
    pluginDirStatus.value = status;
    return status;
  }

  async function initializeDataDir(path: string): Promise<DataDirChangeResult> {
    const result = await settingsApi.initializeDataDir(path);
    dataDirStatus.value = result.status;
    await loadSettings(true);
    return result;
  }

  async function changeDataDir(path: string, migrateExisting = true): Promise<DataDirChangeResult> {
    const result = await settingsApi.changeDataDir(path, migrateExisting);
    dataDirStatus.value = result.status;
    await loadSettings(true);
    return result;
  }

  async function changePluginDir(
    path: string,
    migrateExisting = true,
  ): Promise<PluginDirChangeResult> {
    const result = await settingsApi.changePluginDir(path, migrateExisting);
    pluginDirStatus.value = result.status;
    await loadSettings(true);
    return result;
  }

  async function saveSettings(newSettings: AppSettings): Promise<void> {
    await settingsApi.save(newSettings);
    syncSettings(newSettings);
  }

  async function saveSettingsWithDiff(newSettings: AppSettings): Promise<SettingsGroup[]> {
    const result = await settingsApi.saveWithDiff(newSettings);
    replaceSettings(result.settings, result.changed_groups);
    return result.changed_groups;
  }

  async function updatePartial(partial: PartialSettings): Promise<SettingsGroup[]> {
    const result = await settingsApi.updatePartial(partial);
    replaceSettings(result.settings, result.changed_groups);
    return result.changed_groups;
  }

  async function setLanguage(language: string): Promise<SettingsGroup[]> {
    return updatePartial({ language });
  }

  async function resetSettings(
    changedGroups: SettingsGroup[] = [
      "Appearance",
      "General",
      "ServerDefaults",
      "Console",
      "Window",
      "Developer",
    ],
  ): Promise<AppSettings> {
    const defaultSettingsResult = await settingsApi.reset();
    return replaceSettings(defaultSettingsResult, changedGroups);
  }

  async function importSettingsJson(
    json: string,
    changedGroups: SettingsGroup[] = [
      "Appearance",
      "General",
      "ServerDefaults",
      "Console",
      "Window",
      "Developer",
    ],
  ): Promise<AppSettings> {
    const importedSettings = await settingsApi.importJson(json);
    return replaceSettings(importedSettings, changedGroups);
  }

  async function importPersonalizationPackage(path: string): Promise<ImportPersonalizationResult> {
    const result = await settingsApi.importPersonalizationPackage(path);
    replaceSettings(result.settings, result.changed_groups);
    return result;
  }

  async function setCloseAction(closeAction: string): Promise<SettingsGroup[]> {
    return updatePartial({ close_action: closeAction });
  }

  function updateSettings(partial: Partial<AppSettings>): void {
    settings.value = { ...settings.value, ...partial };
  }

  function getEffectiveTheme(): "light" | "dark" {
    const t = settings.value.theme || "auto";
    if (t === "auto") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
    return t as "light" | "dark";
  }

  return {
    settings,
    dataDirStatus,
    pluginDirStatus,
    isLoaded,
    isLoading,
    loadError,
    theme,
    fontSize,
    windowEffect,
    colorScheme,
    minimalMode,
    backgroundImage,
    backgroundOpacity,
    backgroundBlur,
    backgroundBrightness,
    backgroundSize,
    ensureLoaded,
    loadSettings,
    saveSettings,
    saveSettingsWithDiff,
    updatePartial,
    refreshDataDirStatus,
    refreshPluginDirStatus,
    initializeDataDir,
    changeDataDir,
    changePluginDir,
    setLanguage,
    setCloseAction,
    resetSettings,
    exportSettingsJson,
    getPersonalizationPackageSuggestedName,
    exportPersonalizationPackage,
    importPersonalizationPackage,
    importSettingsJson,
    updateSettings,
    cloneSettings,
    replaceSettings,
    applyClientSettings,
    queueClientSettingsApply,
    getEffectiveTheme,
  };
});
