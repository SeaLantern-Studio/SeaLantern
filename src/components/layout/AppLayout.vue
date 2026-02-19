<script setup lang="ts">
import { onMounted, onUnmounted, computed, watch } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import { useUiStore } from "../../stores/uiStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { applyAcrylic } from "../../api/settings";
import type { AppSettings } from "../../api/settings";
import { getThemeColors, mapLegacyPlanName, type ColorPlan } from "../../themes";

const ui = useUiStore();
const settingsStore = useSettingsStore();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);

let systemThemeQuery: MediaQueryList | null = null;

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

async function applyAcrylicEffect(enabled: boolean, theme: string) {
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");

  if (!settingsStore.acrylicSupported) {
    return;
  }

  if (enabled) {
    const effectiveTheme = getEffectiveTheme(theme);
    const isDark = effectiveTheme === "dark";
    try {
      await applyAcrylic(true, isDark);
    } catch (e) {
      console.error("Failed to apply acrylic:", e);
    }
  } else {
    try {
      await applyAcrylic(false, false);
    } catch (e) {
      console.error("Failed to clear acrylic:", e);
    }
  }
}

function handleSystemThemeChange() {
  const settings = settingsStore.settings;
  if (settings.theme === "auto") {
    applyTheme("auto");
    if (settings.acrylic_enabled && settingsStore.acrylicSupported) {
      applyAcrylicEffect(true, "auto");
    }
    applyColors(settings);
  }
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

function applyDeveloperMode(enabled: boolean) {
  if (enabled) {
    // 开启开发者模式，移除限制
    document.removeEventListener("contextmenu", blockContextMenu);
    document.removeEventListener("keydown", blockDevTools);
  } else {
    // 关闭开发者模式，添加限制
    document.addEventListener("contextmenu", blockContextMenu);
    document.addEventListener("keydown", blockDevTools);
  }
}

function blockContextMenu(e: Event) {
  e.preventDefault();
}

function blockDevTools(e: KeyboardEvent) {
  // 阻止 F12 键
  if (e.key === "F12") {
    e.preventDefault();
  }
}

async function applyAllSettings() {
  const settings = settingsStore.settings;

  applyTheme(settings.theme || "auto");

  document.documentElement.style.fontSize = (settings.font_size || 14) + "px";

  applyFontFamily(settings.font_family || "");

  if (settingsStore.acrylicSupported) {
    await applyAcrylicEffect(settings.acrylic_enabled, settings.theme || "auto");
  } else {
    document.documentElement.setAttribute("data-acrylic", "false");
  }

  applyColors(settings);



  // 应用开发者模式限制
  applyDeveloperMode(settings.developer_mode || false);

  // Apply senior mode attribute
  if (settings.senior_mode) {
    document.documentElement.setAttribute("data-senior", "true");
  } else {
    document.documentElement.removeAttribute("data-senior");
  }
}

onMounted(async () => {
  await applyAllSettings();

  window.addEventListener("settings-updated", handleSettingsUpdated);

  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  window.removeEventListener("settings-updated", handleSettingsUpdated);
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

async function handleSettingsUpdated() {
  await settingsStore.loadSettings();
  await applyAllSettings();
}

watch(
  () => settingsStore.settings,
  async () => {
    await applyAllSettings();
  },
  { deep: true },
);

const backgroundStyle = computed(() => {
  if (!backgroundImage.value) return {};
  return {
    backgroundImage: `url(${backgroundImage.value})`,
    backgroundSize: backgroundSize.value,
    backgroundPosition: "center",
    backgroundRepeat: "no-repeat",
    opacity: backgroundOpacity.value,
    filter: `blur(${backgroundBlur.value}px) brightness(${backgroundBrightness.value})`,
  };
});

function adjustBrightness(hex: string, percent: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const amt = Math.round(2.55 * percent);
  const R = (num >> 16) + amt;
  const G = ((num >> 8) & 0x00ff) + amt;
  const B = (num & 0x0000ff) + amt;
  return (
    "#" +
    (
      0x1000000 +
      (R < 255 ? (R < 1 ? 0 : R) : 255) * 0x10000 +
      (G < 255 ? (G < 1 ? 0 : G) : 255) * 0x100 +
      (B < 255 ? (B < 1 ? 0 : B) : 255)
    )
      .toString(16)
      .slice(1)
  );
}

function rgbaFromHex(hex: string, alpha: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const R = (num >> 16) & 0xff;
  const G = (num >> 8) & 0xff;
  const B = num & 0xff;
  return `rgba(${R}, ${G}, ${B}, ${alpha})`;
}

function getColorValue(settings: AppSettings, colorType: string, theme: string): string {
  if (!settings) return "";

  const plan = mapLegacyPlanName(theme);

  if (settings.color !== "custom") {
    const themeColors = getThemeColors(settings.color, plan);
    if (themeColors) {
      return themeColors[colorType as keyof typeof themeColors] || "";
    }
    return "";
  }

  const customColor: Record<string, Record<string, string | undefined>> = {
    light: {
      bg: settings.bg_color,
      bgSecondary: settings.bg_secondary_color,
      bgTertiary: settings.bg_tertiary_color,
      primary: settings.primary_color,
      secondary: settings.secondary_color,
      textPrimary: settings.text_primary_color,
      textSecondary: settings.text_secondary_color,
      border: settings.border_color,
    },
    dark: {
      bg: settings.bg_dark,
      bgSecondary: settings.bg_secondary_dark,
      bgTertiary: settings.bg_tertiary_dark,
      primary: settings.primary_dark,
      secondary: settings.secondary_dark,
      textPrimary: settings.text_primary_dark,
      textSecondary: settings.text_secondary_dark,
      border: settings.border_dark,
    },
    light_acrylic: {
      bg: settings.bg_acrylic,
      bgSecondary: settings.bg_secondary_acrylic,
      bgTertiary: settings.bg_tertiary_acrylic,
      primary: settings.primary_acrylic,
      secondary: settings.secondary_acrylic,
      textPrimary: settings.text_primary_acrylic,
      textSecondary: settings.text_secondary_acrylic,
      border: settings.border_acrylic,
    },
    dark_acrylic: {
      bg: settings.bg_dark_acrylic,
      bgSecondary: settings.bg_secondary_dark_acrylic,
      bgTertiary: settings.bg_tertiary_dark_acrylic,
      primary: settings.primary_dark_acrylic,
      secondary: settings.secondary_dark_acrylic,
      textPrimary: settings.text_primary_dark_acrylic,
      textSecondary: settings.text_secondary_dark_acrylic,
      border: settings.border_dark_acrylic,
    },
  };

  const themeColors = customColor[theme];
  if (themeColors) {
    return themeColors[colorType] || "";
  }

  return "";
}

function applyColors(settings: AppSettings) {
  if (!settings) return;

  const effectiveTheme = getEffectiveTheme(settings.theme);
  const isDark = effectiveTheme === "dark";
  const isAcrylic = settings.acrylic_enabled;

  const actualPlan = isDark
    ? isAcrylic
      ? "dark_acrylic"
      : "dark"
    : isAcrylic
      ? "light_acrylic"
      : "light";

  const colors = {
    bg: getColorValue(settings, "bg", actualPlan),
    bgSecondary: getColorValue(settings, "bgSecondary", actualPlan),
    bgTertiary: getColorValue(settings, "bgTertiary", actualPlan),
    primary: getColorValue(settings, "primary", actualPlan),
    secondary: getColorValue(settings, "secondary", actualPlan),
    textPrimary: getColorValue(settings, "textPrimary", actualPlan),
    textSecondary: getColorValue(settings, "textSecondary", actualPlan),
    border: getColorValue(settings, "border", actualPlan),
  };

  document.documentElement.style.setProperty("--sl-bg", colors.bg);
  document.documentElement.style.setProperty("--sl-bg-secondary", colors.bgSecondary);
  document.documentElement.style.setProperty("--sl-bg-tertiary", colors.bgTertiary);
  document.documentElement.style.setProperty("--sl-primary", colors.primary);
  document.documentElement.style.setProperty("--sl-accent", colors.secondary);
  document.documentElement.style.setProperty("--sl-text-primary", colors.textPrimary);
  document.documentElement.style.setProperty("--sl-text-secondary", colors.textSecondary);
  document.documentElement.style.setProperty("--sl-border", colors.border);
  document.documentElement.style.setProperty("--sl-border-light", colors.border);

  let surfaceColor, surfaceHoverColor;
  if (isAcrylic) {
    if (isDark) {
      surfaceColor = "rgba(30, 33, 48, 0.6)";
      surfaceHoverColor = "rgba(40, 44, 62, 0.7)";
    } else {
      surfaceColor = "rgba(255, 255, 255, 0.6)";
      surfaceHoverColor = "rgba(248, 250, 252, 0.7)";
    }
  } else {
    surfaceColor = isDark ? colors.bgSecondary : "#ffffff";
    surfaceHoverColor = isDark ? colors.bgTertiary : colors.bg;
  }
  document.documentElement.style.setProperty("--sl-surface", surfaceColor);
  document.documentElement.style.setProperty("--sl-surface-hover", surfaceHoverColor);

  const primaryLight = isDark
    ? adjustBrightness(colors.primary, 30)
    : adjustBrightness(colors.primary, 20);
  const primaryDark = isDark
    ? adjustBrightness(colors.primary, -20)
    : adjustBrightness(colors.primary, -30);
  const primaryBg = isDark ? rgbaFromHex(colors.primary, 0.12) : rgbaFromHex(colors.primary, 0.08);
  document.documentElement.style.setProperty("--sl-primary-light", primaryLight);
  document.documentElement.style.setProperty("--sl-primary-dark", primaryDark);
  document.documentElement.style.setProperty("--sl-primary-bg", primaryBg);

  const accentLight = adjustBrightness(colors.secondary, 20);
  document.documentElement.style.setProperty("--sl-accent-light", accentLight);

  const textTertiary = isDark
    ? adjustBrightness(colors.textSecondary, -20)
    : adjustBrightness(colors.textSecondary, 20);
  const textInverse = "#ffffff";
  document.documentElement.style.setProperty("--sl-text-tertiary", textTertiary);
  document.documentElement.style.setProperty("--sl-text-inverse", textInverse);

  const shadowOpacity = isDark ? 0.4 : 0.06;
  document.documentElement.style.setProperty(
    "--sl-shadow-sm",
    `0 1px 2px rgba(0, 0, 0, ${shadowOpacity * 0.6})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-md",
    `0 4px 12px rgba(0, 0, 0, ${shadowOpacity})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-lg",
    `0 8px 24px rgba(0, 0, 0, ${shadowOpacity * 1.3})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-xl",
    `0 16px 48px rgba(0, 0, 0, ${shadowOpacity * 1.6})`,
  );
}
</script>

<template>
  <div class="app-layout">
    <div class="app-background" :style="backgroundStyle"></div>
    <AppSidebar />
    <div class="app-main" :class="{ 'sidebar-collapsed': ui.sidebarCollapsed }">
      <AppHeader />
      <main class="app-content">
        <router-view v-slot="{ Component }">
          <transition name="page-fade" mode="out-in">
            <keep-alive :max="5">
              <component :is="Component" />
            </keep-alive>
          </transition>
        </router-view>
      </main>
    </div>

  </div>
</template>

<style scoped>
.app-layout {
  position: relative;
  display: flex;
  width: 100vw;
  height: 100vh;
  background-color: var(--sl-bg);
  overflow: hidden;
}

.app-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 0;
  pointer-events: none;
  transition: all 0.3s ease;
}

.app-main {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  margin-left: var(--sl-sidebar-width);
  transition: margin-left 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  min-width: 0;
}

.app-main.sidebar-collapsed {
  margin-left: var(--sl-sidebar-collapsed-width);
}

.app-content {
  flex: 1;
  padding: var(--sl-space-lg);
  overflow-y: auto;
  overflow-x: hidden;
}

.page-fade-enter-active,
.page-fade-leave-active {
  transition:
    opacity 0.15s cubic-bezier(0.4, 0, 0.2, 1),
    transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(4px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-2px);
}
</style>
