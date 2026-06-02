import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  settingsApi,
  type AppSettings,
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
  default_jvm_args: "",
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
    if (typeof structuredClone === "function") {
      return structuredClone(source);
    }
    return JSON.parse(JSON.stringify(source)) as AppSettings;
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
    const enabled = effect !== "off" || !nextSettings.background_image;
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

  async function loadSettings(): Promise<void> {
    if (loadPromise) {
      return loadPromise;
    }

    isLoading.value = true;
    loadError.value = null;

    loadPromise = (async () => {
      try {
        const loadedSettings = await settingsApi.get();
        syncSettings(loadedSettings);
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

  async function exportSettingsJson(): Promise<string> {
    return settingsApi.exportJson();
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
    setLanguage,
    setCloseAction,
    resetSettings,
    exportSettingsJson,
    importSettingsJson,
    updateSettings,
    cloneSettings,
    replaceSettings,
    applyClientSettings,
    queueClientSettingsApply,
    getEffectiveTheme,
  };
});
