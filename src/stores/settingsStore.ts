import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { settingsApi, checkAcrylicSupport, applyAcrylic, type AppSettings } from "../api/settings";
import { convertFileSrc } from "@tauri-apps/api/core";

const THEME_CACHE_KEY = "sl_theme_cache";

function getThemeCache(): { theme: string; fontSize: number } | null {
  try {
    const cached = localStorage.getItem(THEME_CACHE_KEY);
    if (cached) {
      return JSON.parse(cached);
    }
  } catch (e) { }
  return null;
}

function saveThemeCache(theme: string, fontSize: number): void {
  try {
    localStorage.setItem(THEME_CACHE_KEY, JSON.stringify({ theme, fontSize }));
  } catch (e) { }
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
  auto_accept_eula: false,
  default_max_memory: 4096,
  default_min_memory: 1024,
  default_port: 25565,
  default_java_path: "",
  default_jvm_args: "",
  console_font_size: 12,
  max_log_lines: 1000,
  cached_java_list: [],
  background_image: "",
  background_opacity: 0.3,
  background_blur: 0,
  background_brightness: 1.0,
  background_size: "cover",
  acrylic_enabled: false,
  theme: "auto",
  font_size: 14,
  font_family: "",
  color: "default",
  language: "zh-CN",
  developer_mode: false,
  close_action: "ask",
  senior_mode: false,
};

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(defaultSettings);
  const isLoaded = ref(false);
  const isLoading = ref(false);
  const loadError = ref<string | null>(null);
  const acrylicSupported = ref(false);

  const theme = computed(() => settings.value.theme || "auto");
  const fontSize = computed(() => settings.value.font_size || 14);
  const acrylicEnabled = computed(() => settings.value.acrylic_enabled);
  const colorScheme = computed(() => settings.value.color || "default");
  const backgroundImage = computed(() =>
    settings.value.background_image ? convertFileSrc(settings.value.background_image) : "",
  );
  const backgroundOpacity = computed(() => settings.value.background_opacity);
  const backgroundBlur = computed(() => settings.value.background_blur);
  const backgroundBrightness = computed(() => settings.value.background_brightness);
  const backgroundSize = computed(() => settings.value.background_size);

  async function loadSettings(): Promise<void> {
    if (isLoading.value) return;

    isLoading.value = true;
    loadError.value = null;

    try {
      const [loadedSettings, supported] = await Promise.all([
        settingsApi.get(),
        checkAcrylicSupport().catch(() => false),
      ]);

      settings.value = loadedSettings;
      acrylicSupported.value = supported;
      isLoaded.value = true;
      saveThemeCache(loadedSettings.theme || "auto", loadedSettings.font_size || 14);
    } catch (e) {
      console.error("Failed to load settings:", e);
      loadError.value = e instanceof Error ? e.message : String(e);
      settings.value = defaultSettings;
      isLoaded.value = true;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings(newSettings: AppSettings): Promise<void> {
    await settingsApi.save(newSettings);
    settings.value = newSettings;
    saveThemeCache(newSettings.theme || "auto", newSettings.font_size || 14);
  }

  async function resetSettings(): Promise<void> {
    const defaultSettingsResult = await settingsApi.reset();
    settings.value = defaultSettingsResult;
    saveThemeCache(defaultSettingsResult.theme || "auto", defaultSettingsResult.font_size || 14);
  }

  function updateSettings(partial: Partial<AppSettings>): void {
    settings.value = { ...settings.value, ...partial };
  }

  async function applyAcrylicEffect(enabled: boolean): Promise<void> {
    if (!acrylicSupported.value) return;

    const effectiveTheme = getEffectiveTheme();
    const isDark = effectiveTheme === "dark";

    try {
      await applyAcrylic(enabled, isDark);
    } catch (e) {
      console.error("Failed to apply acrylic:", e);
    }
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
    acrylicSupported,
    theme,
    fontSize,
    acrylicEnabled,
    colorScheme,
    backgroundImage,
    backgroundOpacity,
    backgroundBlur,
    backgroundBrightness,
    backgroundSize,
    loadSettings,
    saveSettings,
    resetSettings,
    updateSettings,
    applyAcrylicEffect,
    getEffectiveTheme,
  };
});
