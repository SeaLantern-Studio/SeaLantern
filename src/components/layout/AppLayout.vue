<script setup lang="ts">
import { onMounted, onUnmounted, computed, watch } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import ZombieOverlay from "../effects/ZombieOverlay.vue";
import { useUiStore } from "../../stores/uiStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { applyAcrylic } from "../../api/settings";
import type { AppSettings } from "../../api/settings";

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

  // Apply zombie mode specific classes/attributes
  if (settings.color === "zombie") {
    document.documentElement.setAttribute("data-zombie", "true");
  } else {
    document.documentElement.removeAttribute("data-zombie");
  }

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

const presetThemes = {
  midnight: {
    light: {
      bg: "#f0f4f8",
      bgSecondary: "#e2e8f0",
      bgTertiary: "#cbd5e1",
      primary: "#3b82f6",
      secondary: "#6366f1",
      textPrimary: "#0f172a",
      textSecondary: "#475569",
      border: "#e2e8f0",
    },
    dark: {
      bg: "#0f172a",
      bgSecondary: "#1e293b",
      bgTertiary: "#334155",
      primary: "#60a5fa",
      secondary: "#818cf8",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
    lightAcrylic: {
      bg: "rgba(240, 244, 248, 0.7)",
      bgSecondary: "rgba(226, 232, 240, 0.6)",
      bgTertiary: "rgba(203, 213, 225, 0.5)",
      primary: "#3b82f6",
      secondary: "#6366f1",
      textPrimary: "#0f172a",
      textSecondary: "#475569",
      border: "#e2e8f0",
    },
    darkAcrylic: {
      bg: "rgba(15, 23, 42, 0.7)",
      bgSecondary: "rgba(30, 41, 59, 0.6)",
      bgTertiary: "rgba(51, 65, 85, 0.5)",
      primary: "#60a5fa",
      secondary: "#818cf8",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
  forest: {
    light: {
      bg: "#f0fdf4",
      bgSecondary: "#dcfce7",
      bgTertiary: "#bbf7d0",
      primary: "#10b981",
      secondary: "#059669",
      textPrimary: "#064e3b",
      textSecondary: "#15803d",
      border: "#dcfce7",
    },
    dark: {
      bg: "#064e3b",
      bgSecondary: "#065f46",
      bgTertiary: "#047857",
      primary: "#34d399",
      secondary: "#10b981",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
    lightAcrylic: {
      bg: "rgba(240, 253, 244, 0.7)",
      bgSecondary: "rgba(220, 252, 231, 0.6)",
      bgTertiary: "rgba(187, 247, 208, 0.5)",
      primary: "#10b981",
      secondary: "#059669",
      textPrimary: "#064e3b",
      textSecondary: "#15803d",
      border: "#dcfce7",
    },
    darkAcrylic: {
      bg: "rgba(6, 78, 59, 0.7)",
      bgSecondary: "rgba(6, 95, 70, 0.6)",
      bgTertiary: "rgba(4, 120, 87, 0.5)",
      primary: "#34d399",
      secondary: "#10b981",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
  sunset: {
    light: {
      bg: "#fffbeb",
      bgSecondary: "#fef3c7",
      bgTertiary: "#fde68a",
      primary: "#f97316",
      secondary: "#ea580c",
      textPrimary: "#7c2d12",
      textSecondary: "#9a3412",
      border: "#fef3c7",
    },
    dark: {
      bg: "#7c2d12",
      bgSecondary: "#9a3412",
      bgTertiary: "#b45309",
      primary: "#fb923c",
      secondary: "#fdba74",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
    lightAcrylic: {
      bg: "rgba(255, 251, 235, 0.7)",
      bgSecondary: "rgba(254, 243, 199, 0.6)",
      bgTertiary: "rgba(253, 230, 138, 0.5)",
      primary: "#f97316",
      secondary: "#ea580c",
      textPrimary: "#7c2d12",
      textSecondary: "#9a3412",
      border: "#fef3c7",
    },
    darkAcrylic: {
      bg: "rgba(124, 45, 18, 0.7)",
      bgSecondary: "rgba(154, 52, 18, 0.6)",
      bgTertiary: "rgba(180, 83, 9, 0.5)",
      primary: "#fb923c",
      secondary: "#fdba74",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
  ocean: {
    light: {
      bg: "#f0fdfa",
      bgSecondary: "#ccfbf1",
      bgTertiary: "#99f6e4",
      primary: "#06b6d4",
      secondary: "#0891b2",
      textPrimary: "#0e7490",
      textSecondary: "#155e75",
      border: "#ccfbf1",
    },
    dark: {
      bg: "#0e7490",
      bgSecondary: "#0c4a6e",
      bgTertiary: "#0891b2",
      primary: "#22d3ee",
      secondary: "#67e8f9",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
    lightAcrylic: {
      bg: "rgba(240, 253, 250, 0.7)",
      bgSecondary: "rgba(204, 251, 241, 0.6)",
      bgTertiary: "rgba(153, 246, 228, 0.5)",
      primary: "#06b6d4",
      secondary: "#0891b2",
      textPrimary: "#0e7490",
      textSecondary: "#155e75",
      border: "#ccfbf1",
    },
    darkAcrylic: {
      bg: "rgba(14, 116, 144, 0.7)",
      bgSecondary: "rgba(12, 74, 110, 0.6)",
      bgTertiary: "rgba(8, 145, 178, 0.5)",
      primary: "#22d3ee",
      secondary: "#67e8f9",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
  rose: {
    light: {
      bg: "#fdf2f8",
      bgSecondary: "#fce7f3",
      bgTertiary: "#fbcfe8",
      primary: "#ec4899",
      secondary: "#db2777",
      textPrimary: "#831843",
      textSecondary: "#9f1239",
      border: "#fce7f3",
    },
    dark: {
      bg: "#831843",
      bgSecondary: "#9f1239",
      bgTertiary: "#be123c",
      primary: "#f472b6",
      secondary: "#f9a8d4",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
    lightAcrylic: {
      bg: "rgba(253, 242, 248, 0.7)",
      bgSecondary: "rgba(252, 231, 243, 0.6)",
      bgTertiary: "rgba(251, 207, 232, 0.5)",
      primary: "#ec4899",
      secondary: "#db2777",
      textPrimary: "#831843",
      textSecondary: "#9f1239",
      border: "#fce7f3",
    },
    darkAcrylic: {
      bg: "rgba(131, 24, 67, 0.7)",
      bgSecondary: "rgba(159, 18, 57, 0.6)",
      bgTertiary: "rgba(190, 18, 60, 0.5)",
      primary: "#f472b6",
      secondary: "#f9a8d4",
      textPrimary: "#f1f5f9",
      textSecondary: "#cbd5e1",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
  zombie: {
    light: {
      bg: "#0A0A0A",
      bgSecondary: "#111111",
      bgTertiary: "#1A1A1A",
      primary: "#C8FFB0",
      secondary: "#8B0000",
      textPrimary: "#C8FFB0",
      textSecondary: "#6B8F5E",
      border: "rgba(139, 0, 0, 0.5)",
    },
    dark: {
      bg: "#0A0A0A",
      bgSecondary: "#111111",
      bgTertiary: "#1A1A1A",
      primary: "#C8FFB0",
      secondary: "#8B0000",
      textPrimary: "#C8FFB0",
      textSecondary: "#6B8F5E",
      border: "rgba(139, 0, 0, 0.5)",
    },
    lightAcrylic: {
      bg: "rgba(10, 10, 10, 0.8)",
      bgSecondary: "rgba(17, 17, 17, 0.7)",
      bgTertiary: "rgba(26, 26, 26, 0.6)",
      primary: "#C8FFB0",
      secondary: "#8B0000",
      textPrimary: "#C8FFB0",
      textSecondary: "#6B8F5E",
      border: "rgba(139, 0, 0, 0.5)",
    },
    darkAcrylic: {
      bg: "rgba(10, 10, 10, 0.8)",
      bgSecondary: "rgba(17, 17, 17, 0.7)",
      bgTertiary: "rgba(26, 26, 26, 0.6)",
      primary: "#C8FFB0",
      secondary: "#8B0000",
      textPrimary: "#C8FFB0",
      textSecondary: "#6B8F5E",
      border: "rgba(139, 0, 0, 0.5)",
    },
  },
};

function getPresetColor(preset: string, plan: string, colorType: string): string {
  if (!presetThemes[preset as keyof typeof presetThemes]) return "";

  let themeKey: keyof typeof presetThemes.midnight;
  switch (plan) {
    case "light":
      themeKey = "light";
      break;
    case "dark":
      themeKey = "dark";
      break;
    case "light_acrylic":
      themeKey = "lightAcrylic";
      break;
    case "dark_acrylic":
      themeKey = "darkAcrylic";
      break;
    default:
      return "";
  }

  const presetTheme = presetThemes[preset as keyof typeof presetThemes];
  return presetTheme[themeKey][colorType as keyof typeof presetTheme.light] || "";
}

function getColorValue(settings: AppSettings, colorType: string, theme: string): string {
  if (!settings) return "";

  if (settings.color !== "custom") {
    if (settings.color === "default") {
      switch (theme) {
        case "light":
          return (
            {
              bg: "#f8fafc",
              bgSecondary: "#f1f5f9",
              bgTertiary: "#e2e8f0",
              primary: "#0ea5e9",
              secondary: "#06b6d4",
              textPrimary: "#0f172a",
              textSecondary: "#475569",
              border: "#e2e8f0",
            }[colorType] || ""
          );
        case "dark":
          return (
            {
              bg: "#0f1117",
              bgSecondary: "#1a1d28",
              bgTertiary: "#242836",
              primary: "#60a5fa",
              secondary: "#22d3ee",
              textPrimary: "#e2e8f0",
              textSecondary: "#94a3b8",
              border: "rgba(255, 255, 255, 0.1)",
            }[colorType] || ""
          );
        case "light_acrylic":
          return (
            {
              bg: "rgba(248, 250, 252, 0.7)",
              bgSecondary: "rgba(241, 245, 249, 0.6)",
              bgTertiary: "rgba(226, 232, 240, 0.5)",
              primary: "#0ea5e9",
              secondary: "#06b6d4",
              textPrimary: "#0f172a",
              textSecondary: "#475569",
              border: "#e2e8f0",
            }[colorType] || ""
          );
        case "dark_acrylic":
          return (
            {
              bg: "rgba(15, 17, 23, 0.7)",
              bgSecondary: "rgba(26, 29, 40, 0.6)",
              bgTertiary: "rgba(36, 40, 54, 0.5)",
              primary: "#60a5fa",
              secondary: "#22d3ee",
              textPrimary: "#e2e8f0",
              textSecondary: "#94a3b8",
              border: "rgba(255, 255, 255, 0.1)",
            }[colorType] || ""
          );
        default:
          return "";
      }
    }
    return getPresetColor(settings.color, theme, colorType);
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
    <ZombieOverlay v-if="settingsStore.settings.color === 'zombie'" />
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
