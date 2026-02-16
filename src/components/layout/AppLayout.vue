<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import { useUiStore } from "../../stores/uiStore";
import {
  settingsApi,
  applyAcrylic,
  checkAcrylicSupport,
  type AppSettings,
} from "../../api/settings";
import { convertFileSrc } from "@tauri-apps/api/core";

const ui = useUiStore();
const backgroundImage = ref("");
const backgroundOpacity = ref(0.3);
const backgroundBlur = ref(0);
const backgroundBrightness = ref(1.0);
const backgroundSize = ref("cover");
const acrylicEnabled = ref(false);
const acrylicSupported = ref(false);
const currentColor = ref("default");
const currentTheme = ref("auto");

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

  if (!acrylicSupported.value) {
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

// 系统主题变化处理
function handleSystemThemeChange() {
  if (currentTheme.value === "auto") {
    const effectiveTheme = applyTheme("auto");
    // 如果亚克力开启，需要重新应用以匹配新主题
    if (acrylicEnabled.value && acrylicSupported.value) {
      applyAcrylicEffect(true, "auto");
    }
  }
}

onMounted(() => {
  // 立即应用默认主题，确保 UI 快速显示
  applyTheme("auto");

  // 检测亚克力支持（异步，不阻塞 UI）
  checkAcrylicSupport()
    .then((supported) => {
      acrylicSupported.value = supported;
    })
    .catch(() => {
      acrylicSupported.value = false;
    });

  // 异步加载背景设置，不阻塞 UI 渲染
  loadBackgroundSettings().catch((err) => {
    console.error("Failed to load settings initially:", err);
  });

  // 监听设置更新事件
  window.addEventListener("settings-updated", loadBackgroundSettings);

  // 监听系统主题变化
  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  window.removeEventListener("settings-updated", loadBackgroundSettings);
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

async function loadBackgroundSettings() {
  try {
    const settings = await settingsApi.get();

    // 应用颜色方案
    currentColor.value = settings.color || "default";

    // 保存当前主题设置
    currentTheme.value = settings.theme || "auto";
    acrylicEnabled.value = settings.acrylic_enabled;

    // 应用主题
    const effectiveTheme = applyTheme(settings.theme || "auto");

    // 应用字体大小
    document.documentElement.style.fontSize = (settings.font_size || 14) + "px";

    // 应用亚克力效果（只有在支持的系统上）
    if (acrylicSupported.value) {
      await applyAcrylicEffect(settings.acrylic_enabled, settings.theme || "auto");
    } else {
      // 不支持亚克力时，确保 data-acrylic 为 false
      document.documentElement.setAttribute("data-acrylic", "false");
    }

    // 应用背景图片设置
    if (settings.background_image) {
      backgroundImage.value = convertFileSrc(settings.background_image);
    } else {
      backgroundImage.value = "";
    }
    backgroundOpacity.value = settings.background_opacity;
    backgroundBlur.value = settings.background_blur;
    backgroundBrightness.value = settings.background_brightness;
    backgroundSize.value = settings.background_size;

    // 应用颜色主题
    applyColors(settings);

    console.log("Settings loaded:", {
      theme: settings.theme,
      effectiveTheme,
      fontSize: settings.font_size,
      acrylicEnabled: settings.acrylic_enabled,
      acrylicSupported: acrylicSupported.value,
      image: backgroundImage.value,
      opacity: backgroundOpacity.value,
      blur: backgroundBlur.value,
      brightness: backgroundBrightness.value,
      size: backgroundSize.value,
    });
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

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

// 辅助函数：调整颜色亮度
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

// 辅助函数：将十六进制颜色转换为 RGBA
function rgbaFromHex(hex: string, alpha: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const R = (num >> 16) & 0xff;
  const G = (num >> 8) & 0xff;
  const B = num & 0xff;
  return `rgba(${R}, ${G}, ${B}, ${alpha})`;
}

// 预设主题颜色定义
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
};

// 获取预设主题颜色
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

// 获取颜色值
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

  const customColor = {
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
  }[theme as keyof typeof customColor];

  if (customColor) {
    return customColor[colorType as keyof typeof customColor.light] || "";
  }

  // 如果没有自定义颜色，使用当前预设的颜色值
  if (settings.color_prev) {
    if (settings.color_prev === "default") {
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
    return getPresetColor(settings.color_prev, theme, colorType);
  }

  return "";
}

// 应用颜色
function applyColors(settings: AppSettings) {
  if (!settings) return;

  // 确定当前的主题模式
  const effectiveTheme = getEffectiveTheme(settings.theme);
  const isDark = effectiveTheme === "dark";

  // 确定当前是否启用了亚克力
  const isAcrylic = settings.acrylic_enabled;

  // 根据实际情况确定当前的颜色方案
  const actualPlan = isDark
    ? isAcrylic
      ? "dark_acrylic"
      : "dark"
    : isAcrylic
      ? "light_acrylic"
      : "light";

  // 获取当前的颜色值
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

  // 应用颜色值到 CSS 变量
  document.documentElement.style.setProperty("--sl-bg", colors.bg);
  document.documentElement.style.setProperty("--sl-bg-secondary", colors.bgSecondary);
  document.documentElement.style.setProperty("--sl-bg-tertiary", colors.bgTertiary);
  document.documentElement.style.setProperty("--sl-primary", colors.primary);
  document.documentElement.style.setProperty("--sl-accent", colors.secondary);
  document.documentElement.style.setProperty("--sl-text-primary", colors.textPrimary);
  document.documentElement.style.setProperty("--sl-text-secondary", colors.textSecondary);
  document.documentElement.style.setProperty("--sl-border", colors.border);
  document.documentElement.style.setProperty("--sl-border-light", colors.border);

  // 计算并应用其他相关变量

  // 表面颜色
  let surfaceColor, surfaceHoverColor;
  if (isAcrylic) {
    // 参考 variables.css 为亚克力方案设置 rgba 颜色
    if (isDark) {
      surfaceColor = "rgba(30, 33, 48, 0.6)";
      surfaceHoverColor = "rgba(40, 44, 62, 0.7)";
    } else {
      surfaceColor = "rgba(255, 255, 255, 0.6)";
      surfaceHoverColor = "rgba(248, 250, 252, 0.7)";
    }
  } else {
    // 非亚克力方案使用原来的颜色
    surfaceColor = isDark ? colors.bgSecondary : "#ffffff";
    surfaceHoverColor = isDark ? colors.bgTertiary : colors.bg;
  }
  document.documentElement.style.setProperty("--sl-surface", surfaceColor);
  document.documentElement.style.setProperty("--sl-surface-hover", surfaceHoverColor);

  // 主要颜色变体
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

  // 强调色变体
  const accentLight = adjustBrightness(colors.secondary, 20);
  document.documentElement.style.setProperty("--sl-accent-light", accentLight);

  // 文本颜色变体
  const textTertiary = isDark
    ? adjustBrightness(colors.textSecondary, -20)
    : adjustBrightness(colors.textSecondary, 20);
  // 确保主要按钮、危险按钮和成功按钮的文字颜色始终是白色
  const textInverse = "#ffffff";
  document.documentElement.style.setProperty("--sl-text-tertiary", textTertiary);
  document.documentElement.style.setProperty("--sl-text-inverse", textInverse);

  // 阴影
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
