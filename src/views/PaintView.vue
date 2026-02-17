<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSwitch from "../components/common/SLSwitch.vue";
import SLModal from "../components/common/SLModal.vue";
import SLSelect from "../components/common/SLSelect.vue";
import { i18n } from "../locales";
import {
  settingsApi,
  checkAcrylicSupport,
  applyAcrylic,
  getSystemFonts,
  type AppSettings,
} from "../api/settings";
import { systemApi } from "../api/system";
import { convertFileSrc } from "@tauri-apps/api/core";

// 预设主题颜色定义
const presetThemes = {
  default: {
    light: {
      bg: "#f8fafc",
      bgSecondary: "#f1f5f9",
      bgTertiary: "#e2e8f0",
      primary: "#0ea5e9",
      secondary: "#06b6d4",
      textPrimary: "#0f172a",
      textSecondary: "#475569",
      border: "#e2e8f0",
    },
    dark: {
      bg: "#0f1117",
      bgSecondary: "#1a1d28",
      bgTertiary: "#242836",
      primary: "#60a5fa",
      secondary: "#22d3ee",
      textPrimary: "#e2e8f0",
      textSecondary: "#94a3b8",
      border: "rgba(255, 255, 255, 0.1)",
    },
    light_acrylic: {
      bg: "rgba(248, 250, 252, 0.7)",
      bgSecondary: "rgba(241, 245, 249, 0.6)",
      bgTertiary: "rgba(226, 232, 240, 0.5)",
      primary: "#0ea5e9",
      secondary: "#06b6d4",
      textPrimary: "#0f172a",
      textSecondary: "#475569",
      border: "#e2e8f0",
    },
    dark_acrylic: {
      bg: "rgba(15, 17, 23, 0.7)",
      bgSecondary: "rgba(26, 29, 40, 0.6)",
      bgTertiary: "rgba(36, 40, 54, 0.5)",
      primary: "#60a5fa",
      secondary: "#22d3ee",
      textPrimary: "#e2e8f0",
      textSecondary: "#94a3b8",
      border: "rgba(255, 255, 255, 0.1)",
    },
  },
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
    light_acrylic: {
      bg: "rgba(240, 244, 248, 0.7)",
      bgSecondary: "rgba(226, 232, 240, 0.6)",
      bgTertiary: "rgba(203, 213, 225, 0.5)",
      primary: "#3b82f6",
      secondary: "#6366f1",
      textPrimary: "#0f172a",
      textSecondary: "#475569",
      border: "#e2e8f0",
    },
    dark_acrylic: {
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
    light_acrylic: {
      bg: "rgba(240, 253, 244, 0.7)",
      bgSecondary: "rgba(220, 252, 231, 0.6)",
      bgTertiary: "rgba(187, 247, 208, 0.5)",
      primary: "#10b981",
      secondary: "#059669",
      textPrimary: "#064e3b",
      textSecondary: "#15803d",
      border: "#dcfce7",
    },
    dark_acrylic: {
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
    light_acrylic: {
      bg: "rgba(255, 251, 235, 0.7)",
      bgSecondary: "rgba(254, 243, 199, 0.6)",
      bgTertiary: "rgba(253, 230, 138, 0.5)",
      primary: "#f97316",
      secondary: "#ea580c",
      textPrimary: "#7c2d12",
      textSecondary: "#9a3412",
      border: "#fef3c7",
    },
    dark_acrylic: {
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
    light_acrylic: {
      bg: "rgba(240, 253, 250, 0.7)",
      bgSecondary: "rgba(204, 251, 241, 0.6)",
      bgTertiary: "rgba(153, 246, 228, 0.5)",
      primary: "#06b6d4",
      secondary: "#0891b2",
      textPrimary: "#0e7490",
      textSecondary: "#155e75",
      border: "#ccfbf1",
    },
    dark_acrylic: {
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
    light_acrylic: {
      bg: "rgba(253, 242, 248, 0.7)",
      bgSecondary: "rgba(252, 231, 243, 0.6)",
      bgTertiary: "rgba(251, 207, 232, 0.5)",
      primary: "#ec4899",
      secondary: "#db2777",
      textPrimary: "#831843",
      textSecondary: "#9f1239",
      border: "#fce7f3",
    },
    dark_acrylic: {
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

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

// 亚克力支持检测
const acrylicSupported = ref(true);

// String versions for number inputs (avoids v-model type mismatch)
const maxMem = ref("2048");
const minMem = ref("512");
const port = ref("25565");
const fontSize = ref("13");
const logLines = ref("5000");
const bgOpacity = ref("0.3");
const bgBlur = ref("0");
const bgBrightness = ref("1.0");
const uiFontSize = ref("14");

const backgroundSizeOptions = computed(() => [
  { label: i18n.t("settings.background_size_options.cover"), value: "cover" },
  { label: i18n.t("settings.background_size_options.contain"), value: "contain" },
  { label: i18n.t("settings.background_size_options.fill"), value: "fill" },
  { label: i18n.t("settings.background_size_options.auto"), value: "auto" },
]);

const colorOptions = computed(() => [
  { label: i18n.t("settings.color_options.default"), value: "default" },
  { label: i18n.t("settings.color_options.midnight"), value: "midnight" },
  { label: i18n.t("settings.color_options.forest"), value: "forest" },
  { label: i18n.t("settings.color_options.sunset"), value: "sunset" },
  { label: i18n.t("settings.color_options.ocean"), value: "ocean" },
  { label: i18n.t("settings.color_options.rose"), value: "rose" },
  { label: i18n.t("settings.color_options.custom"), value: "custom" },
]);

const editColorOptions = computed(() => [
  { label: i18n.t("settings.edit_colorplan_options.light"), value: "light" },
  { label: i18n.t("settings.edit_colorplan_options.dark"), value: "dark" },
  { label: i18n.t("settings.edit_colorplan_options.light_acrylic"), value: "light_acrylic" },
  { label: i18n.t("settings.edit_colorplan_options.dark_acrylic"), value: "dark_acrylic" },
]);

const editColorPlan = ref<"light" | "dark" | "light_acrylic" | "dark_acrylic">("light");

const colorSchemes: Record<
  string,
  {
    bg: string;
    bgSecondary: string;
    bgTertiary: string;
    primary: string;
    secondary: string;
    textPrimary: string;
    textSecondary: string;
    border: string;
  }
> = {
  light: {
    bg: "#f8fafc",
    bgSecondary: "#f1f5f9",
    bgTertiary: "#e2e8f0",
    primary: "#0ea5e9",
    secondary: "#06b6d4",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  dark: {
    bg: "#0f1117",
    bgSecondary: "#1a1d28",
    bgTertiary: "#242836",
    primary: "#60a5fa",
    secondary: "#22d3ee",
    textPrimary: "#e2e8f0",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.1)",
  },
  light_acrylic: {
    bg: "rgba(248, 250, 252, 0.7)",
    bgSecondary: "rgba(241, 245, 249, 0.6)",
    bgTertiary: "rgba(226, 232, 240, 0.5)",
    primary: "#0ea5e9",
    secondary: "#06b6d4",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  dark_acrylic: {
    bg: "rgba(15, 17, 23, 0.7)",
    bgSecondary: "rgba(26, 29, 40, 0.6)",
    bgTertiary: "rgba(36, 40, 54, 0.5)",
    primary: "#60a5fa",
    secondary: "#22d3ee",
    textPrimary: "#e2e8f0",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

const themeOptions = [
  { label: i18n.t("settings.theme_options.auto"), value: "auto" },
  { label: i18n.t("settings.theme_options.light"), value: "light" },
  { label: i18n.t("settings.theme_options.dark"), value: "dark" },
];

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t("settings.font_family_default"), value: "" },
]);

const showImportModal = ref(false);
const importJson = ref("");
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
const bgPreviewLoaded = ref(false);
const bgPreviewLoading = ref(false);
const colorSettingsExpanded = ref(false);

const backgroundPreviewUrl = computed(() => {
  if (!settings.value?.background_image) return "";
  if (!bgSettingsExpanded.value) return "";
  return convertFileSrc(settings.value.background_image);
});

const bgColor = computed({
  get: () => {
    if (!settings.value) return "#f8fafc";

    const colorMap = {
      light: settings.value.bg_color,
      dark: settings.value.bg_dark,
      light_acrylic: settings.value.bg_acrylic,
      dark_acrylic: settings.value.bg_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#f8fafc";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "bg_color",
      dark: "bg_dark",
      light_acrylic: "bg_acrylic",
      dark_acrylic: "bg_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const bgSecondaryColor = computed({
  get: () => {
    if (!settings.value) return "#ffffff";

    const colorMap = {
      light: settings.value.bg_secondary_color,
      dark: settings.value.bg_secondary_dark,
      light_acrylic: settings.value.bg_secondary_acrylic,
      dark_acrylic: settings.value.bg_secondary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#ffffff";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "bg_secondary_color",
      dark: "bg_secondary_dark",
      light_acrylic: "bg_secondary_acrylic",
      dark_acrylic: "bg_secondary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const bgTertiaryColor = computed({
  get: () => {
    if (!settings.value) return "#f1f5f9";

    const colorMap = {
      light: settings.value.bg_tertiary_color,
      dark: settings.value.bg_tertiary_dark,
      light_acrylic: settings.value.bg_tertiary_acrylic,
      dark_acrylic: settings.value.bg_tertiary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#f1f5f9";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "bg_tertiary_color",
      dark: "bg_tertiary_dark",
      light_acrylic: "bg_tertiary_acrylic",
      dark_acrylic: "bg_tertiary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const primaryColor = computed({
  get: () => {
    if (!settings.value) return "#3b82f6";

    const colorMap = {
      light: settings.value.primary_color,
      dark: settings.value.primary_dark,
      light_acrylic: settings.value.primary_acrylic,
      dark_acrylic: settings.value.primary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#3b82f6";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "primary_color",
      dark: "primary_dark",
      light_acrylic: "primary_acrylic",
      dark_acrylic: "primary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const secondaryColor = computed({
  get: () => {
    if (!settings.value) return "#8b5cf6";

    const colorMap = {
      light: settings.value.secondary_color,
      dark: settings.value.secondary_dark,
      light_acrylic: settings.value.secondary_acrylic,
      dark_acrylic: settings.value.secondary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#8b5cf6";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "secondary_color",
      dark: "secondary_dark",
      light_acrylic: "secondary_acrylic",
      dark_acrylic: "secondary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const textPrimaryColor = computed({
  get: () => {
    if (!settings.value) return "#1e293b";

    const colorMap = {
      light: settings.value.text_primary_color,
      dark: settings.value.text_primary_dark,
      light_acrylic: settings.value.text_primary_acrylic,
      dark_acrylic: settings.value.text_primary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#1e293b";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "text_primary_color",
      dark: "text_primary_dark",
      light_acrylic: "text_primary_acrylic",
      dark_acrylic: "text_primary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const textSecondaryColor = computed({
  get: () => {
    if (!settings.value) return "#64748b";

    const colorMap = {
      light: settings.value.text_secondary_color,
      dark: settings.value.text_secondary_dark,
      light_acrylic: settings.value.text_secondary_acrylic,
      dark_acrylic: settings.value.text_secondary_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#64748b";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "text_secondary_color",
      dark: "text_secondary_dark",
      light_acrylic: "text_secondary_acrylic",
      dark_acrylic: "text_secondary_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

const borderColor = computed({
  get: () => {
    if (!settings.value) return "#e2e8f0";

    const colorMap = {
      light: settings.value.border_color,
      dark: settings.value.border_dark,
      light_acrylic: settings.value.border_acrylic,
      dark_acrylic: settings.value.border_dark_acrylic,
    };

    return colorMap[editColorPlan.value as keyof typeof colorMap] || "#e2e8f0";
  },
  set: (value) => {
    if (!settings.value) return;

    const colorMap = {
      light: "border_color",
      dark: "border_dark",
      light_acrylic: "border_acrylic",
      dark_acrylic: "border_dark_acrylic",
    };

    const colorKey = colorMap[editColorPlan.value as keyof typeof colorMap];
    if (colorKey) {
      (settings.value as any)[colorKey] = value;
      settings.value.color = "custom";
      markChanged();
      applyColors();
    }
  },
});

// 颜色选择器状态
const showColorPickerDialog = ref(false);
const currentColorProp = ref("");
const currentColorValue = ref("");
const originalColorValue = ref("");
const rgb = ref({ r: 0, g: 0, b: 0, a: 1 });
const hsl = ref({ h: 0, s: 0, l: 0 });

// 显示颜色选择器
function showColorPicker(prop: string) {
  currentColorProp.value = prop;

  // 根据属性名获取当前颜色值
  switch (prop) {
    case "bgColor":
      currentColorValue.value = bgColor.value;
      originalColorValue.value = bgColor.value;
      break;
    case "bgSecondaryColor":
      currentColorValue.value = bgSecondaryColor.value;
      originalColorValue.value = bgSecondaryColor.value;
      break;
    case "bgTertiaryColor":
      currentColorValue.value = bgTertiaryColor.value;
      originalColorValue.value = bgTertiaryColor.value;
      break;
    case "primaryColor":
      currentColorValue.value = primaryColor.value;
      originalColorValue.value = primaryColor.value;
      break;
    case "secondaryColor":
      currentColorValue.value = secondaryColor.value;
      originalColorValue.value = secondaryColor.value;
      break;
    case "textPrimaryColor":
      currentColorValue.value = textPrimaryColor.value;
      originalColorValue.value = textPrimaryColor.value;
      break;
    case "textSecondaryColor":
      currentColorValue.value = textSecondaryColor.value;
      originalColorValue.value = textSecondaryColor.value;
      break;
    case "borderColor":
      currentColorValue.value = borderColor.value;
      originalColorValue.value = borderColor.value;
      break;
  }

  // 解析颜色值为 RGB
  parseColor(currentColorValue.value);

  showColorPickerDialog.value = true;
}

// 解析颜色值
function parseColor(color: string) {
  try {
    // 创建临时元素来解析颜色
    const temp = document.createElement("div");
    temp.style.color = color;
    document.body.appendChild(temp);

    // 获取计算后的颜色值
    const computedColor = window.getComputedStyle(temp).color;
    document.body.removeChild(temp);

    // 从 rgba(r, g, b, a) 格式中提取值
    const match =
      color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*(\d+(?:\.\d+)?))?\)/) ||
      computedColor.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*(\d+(?:\.\d+)?))?\)/);
    if (match) {
      rgb.value = {
        r: parseInt(match[1]),
        g: parseInt(match[2]),
        b: parseInt(match[3]),
        a: match[4] ? parseFloat(match[4]) : 1,
      };

      // 转换为 HSL
      rgbToHsl(rgb.value.r, rgb.value.g, rgb.value.b);
    }
  } catch (e) {
    // 如果解析失败，使用默认值
    rgb.value = { r: 0, g: 0, b: 0, a: 1 };
    hsl.value = { h: 0, s: 0, l: 0 };
  }
}

// RGB 转 HSL
function rgbToHsl(r: number, g: number, b: number) {
  r /= 255;
  g /= 255;
  b /= 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }

    h /= 6;
  }

  hsl.value = {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100),
  };
}

// HSL 转 RGB
function hslToRgb(h: number, s: number, l: number) {
  h /= 360;
  s /= 100;
  l /= 100;

  let r, g, b;

  if (s === 0) {
    r = g = b = l; // 灰色
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;

    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }

  return {
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255),
  };
}

// 关闭颜色选择器
function closeColorPicker() {
  showColorPickerDialog.value = false;
}

// 更新颜色值
function updateColor(value: string) {
  currentColorValue.value = value;
  parseColor(value);
}

// 确认颜色选择
function confirmColorPicker() {
  if (currentColorProp.value) {
    const value = currentColorValue.value;
    switch (currentColorProp.value) {
      case "bgColor":
        bgColor.value = value;
        break;
      case "bgSecondaryColor":
        bgSecondaryColor.value = value;
        break;
      case "bgTertiaryColor":
        bgTertiaryColor.value = value;
        break;
      case "primaryColor":
        primaryColor.value = value;
        break;
      case "secondaryColor":
        secondaryColor.value = value;
        break;
      case "textPrimaryColor":
        textPrimaryColor.value = value;
        break;
      case "textSecondaryColor":
        textSecondaryColor.value = value;
        break;
      case "borderColor":
        borderColor.value = value;
        break;
    }
  }
  showColorPickerDialog.value = false;
}

// 取消颜色选择
function cancelColorPicker() {
  // 恢复原始颜色值
  currentColorValue.value = originalColorValue.value;
  parseColor(originalColorValue.value);
  showColorPickerDialog.value = false;
}

// RGB 转 HEX
function rgbToHex(r: number, g: number, b: number): string {
  return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1).toUpperCase()}`;
}

// 从 RGB 值更新颜色
function updateFromRGB() {
  const { r, g, b, a } = rgb.value;
  let colorValue;

  // 如果透明度为 1，使用 HEX 格式，否则使用 RGBA 格式
  if (a === 1) {
    colorValue = rgbToHex(r, g, b);
  } else {
    colorValue = `rgba(${r}, ${g}, ${b}, ${a})`;
  }

  currentColorValue.value = colorValue;
  updateColor(colorValue);
  rgbToHsl(r, g, b);
}

// 从鼠标事件更新色相
function updateHueFromMouseEvent(event: MouseEvent) {
  // 获取滑块轨道元素，确保使用正确的尺寸
  let target = event.currentTarget as HTMLElement;
  // 如果目标是thumb元素，找到其父元素（滑块轨道）
  if (target.classList.contains("hue-thumb")) {
    target = target.parentElement as HTMLElement;
  }
  const rect = target.getBoundingClientRect();
  const offsetX = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(1, offsetX / rect.width));

  hsl.value.h = Math.round(percentage * 360);
  const rgbValues = hslToRgb(hsl.value.h, hsl.value.s, hsl.value.l);
  rgb.value.r = rgbValues.r;
  rgb.value.g = rgbValues.g;
  rgb.value.b = rgbValues.b;

  updateFromRGB();
}

// 从鼠标事件更新饱和度
function updateSaturationFromMouseEvent(event: MouseEvent) {
  // 获取滑块轨道元素，确保使用正确的尺寸
  let target = event.currentTarget as HTMLElement;
  // 如果目标是thumb元素，找到其父元素（滑块轨道）
  if (target.classList.contains("saturation-thumb")) {
    target = target.parentElement as HTMLElement;
  }
  const rect = target.getBoundingClientRect();
  const offsetX = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(1, offsetX / rect.width));

  hsl.value.s = Math.round(percentage * 100);
  const rgbValues = hslToRgb(hsl.value.h, hsl.value.s, hsl.value.l);
  rgb.value.r = rgbValues.r;
  rgb.value.g = rgbValues.g;
  rgb.value.b = rgbValues.b;

  updateFromRGB();
}

// 从鼠标事件更新亮度
function updateLightnessFromMouseEvent(event: MouseEvent) {
  // 获取滑块轨道元素，确保使用正确的尺寸
  let target = event.currentTarget as HTMLElement;
  // 如果目标是thumb元素，找到其父元素（滑块轨道）
  if (target.classList.contains("lightness-thumb")) {
    target = target.parentElement as HTMLElement;
  }
  const rect = target.getBoundingClientRect();
  const offsetX = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(1, offsetX / rect.width));

  hsl.value.l = Math.round(percentage * 100);
  const rgbValues = hslToRgb(hsl.value.h, hsl.value.s, hsl.value.l);
  rgb.value.r = rgbValues.r;
  rgb.value.g = rgbValues.g;
  rgb.value.b = rgbValues.b;

  updateFromRGB();
}

// 从鼠标事件更新透明度
function updateAlphaFromMouseEvent(event: MouseEvent) {
  // 获取滑块轨道元素，确保使用正确的尺寸
  let target = event.currentTarget as HTMLElement;
  // 如果目标是thumb元素，找到其父元素（滑块轨道）
  if (target.classList.contains("alpha-thumb")) {
    target = target.parentElement as HTMLElement;
  }
  const rect = target.getBoundingClientRect();
  const offsetX = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(1, offsetX / rect.width));

  rgb.value.a = parseFloat(percentage.toFixed(2));
  updateFromRGB();
}

// 处理滑块拖动
function handleSliderDrag(startEvent: MouseEvent, updateFunction: (event: MouseEvent) => void) {
  // 阻止默认行为
  startEvent.preventDefault();

  // 获取滑块轨道元素，确保使用正确的尺寸
  let sliderTrack: HTMLElement;
  const startTarget = startEvent.currentTarget as HTMLElement;

  // 如果目标是thumb元素，找到其父元素（滑块轨道）
  if (
    startTarget.classList.contains("hue-thumb") ||
    startTarget.classList.contains("saturation-thumb") ||
    startTarget.classList.contains("lightness-thumb") ||
    startTarget.classList.contains("alpha-thumb")
  ) {
    sliderTrack = startTarget.parentElement as HTMLElement;
  } else {
    sliderTrack = startTarget;
  }

  // 自定义更新函数，确保使用正确的滑块轨道元素
  const customUpdateFunction = (event: MouseEvent) => {
    // 创建一个新的事件对象，覆盖currentTarget为滑块轨道
    const customEvent = new MouseEvent(event.type, event);
    Object.defineProperty(customEvent, "currentTarget", {
      value: sliderTrack,
      writable: false,
    });

    // 调用原始更新函数
    updateFunction(customEvent);
  };

  // 开始拖动时更新一次
  customUpdateFunction(startEvent);

  // 创建鼠标移动和释放的处理函数
  const handleMouseMove = (moveEvent: MouseEvent) => {
    customUpdateFunction(moveEvent);
  };

  const handleMouseUp = () => {
    // 移除事件监听器
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  };

  // 添加事件监听器
  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
}

// 从点击事件更新色相
function updateHueFromClick(event: MouseEvent) {
  updateHueFromMouseEvent(event);
}

// 从点击事件更新饱和度
function updateSaturationFromClick(event: MouseEvent) {
  updateSaturationFromMouseEvent(event);
}

// 从点击事件更新亮度
function updateLightnessFromClick(event: MouseEvent) {
  updateLightnessFromMouseEvent(event);
}

// 从点击事件更新透明度
function updateAlphaFromClick(event: MouseEvent) {
  updateAlphaFromMouseEvent(event);
}

// 处理色相滑块拖动
function handleHueDrag(event: MouseEvent) {
  handleSliderDrag(event, updateHueFromMouseEvent);
}

// 处理饱和度滑块拖动
function handleSaturationDrag(event: MouseEvent) {
  handleSliderDrag(event, updateSaturationFromMouseEvent);
}

// 处理亮度滑块拖动
function handleLightnessDrag(event: MouseEvent) {
  handleSliderDrag(event, updateLightnessFromMouseEvent);
}

// 处理透明度滑块拖动
function handleAlphaDrag(event: MouseEvent) {
  handleSliderDrag(event, updateAlphaFromMouseEvent);
}

function getFileExtension(path: string): string {
  return path.split(".").pop()?.toLowerCase() || "";
}

function isAnimatedImage(path: string): boolean {
  const ext = getFileExtension(path);
  return ext === "gif" || ext === "webp" || ext === "apng";
}

onMounted(async () => {
  await loadSettings();
  await loadSystemFonts();
  // 检测亚克力支持
  try {
    acrylicSupported.value = await checkAcrylicSupport();
  } catch {
    acrylicSupported.value = false;
  }
  // 应用初始颜色
  applyColors();
});

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

watch(bgSettingsExpanded, (expanded) => {
  if (expanded && settings.value?.background_image) {
    bgPreviewLoaded.value = false;
    bgPreviewLoading.value = true;
  }
});

async function loadSettings() {
  loading.value = true;
  error.value = null;
  try {
    const s = await settingsApi.get();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
    settings.value.color = s.color || "default";
    // 应用已保存的设置
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
  saveSettings();
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
  const size = parseInt(uiFontSize.value) || 14;
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

function applyColors() {
  if (!settings.value) return;

  // 确定当前的主题模式
  const effectiveTheme = getEffectiveTheme(settings.value.theme);
  const isDark = effectiveTheme === "dark";

  // 确定当前是否启用了亚克力
  const isAcrylic = settings.value.acrylic_enabled && acrylicSupported.value;

  // 根据实际情况确定当前的颜色方案
  const actualPlan = isDark
    ? isAcrylic
      ? "dark_acrylic"
      : "dark"
    : isAcrylic
      ? "light_acrylic"
      : "light";

  // 保存当前的编辑方案
  const originalPlan = editColorPlan.value;

  // 临时切换到实际的颜色方案
  editColorPlan.value = actualPlan;

  // 获取当前的颜色值
  const colors = {
    bg: bgColor.value,
    bgSecondary: bgSecondaryColor.value,
    bgTertiary: bgTertiaryColor.value,
    primary: primaryColor.value,
    secondary: secondaryColor.value,
    textPrimary: textPrimaryColor.value,
    textSecondary: textSecondaryColor.value,
    border: borderColor.value,
  };

  // 恢复原来的编辑方案
  editColorPlan.value = originalPlan;

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

async function handleThemeChange() {
  if (!settings.value) return;

  const effectiveTheme = applyTheme(settings.value.theme);

  // 如果选择了预设主题，更新颜色值
  if (settings.value.color !== "custom") {
    const preset = settings.value.color;

    // 颜色方案映射
    const colorPlans = ["light", "dark", "light_acrylic", "dark_acrylic"];

    // 颜色类型映射
    const colorTypes = {
      bg: {
        light: "bg_color",
        dark: "bg_dark",
        light_acrylic: "bg_acrylic",
        dark_acrylic: "bg_dark_acrylic",
      },
      bgSecondary: {
        light: "bg_secondary_color",
        dark: "bg_secondary_dark",
        light_acrylic: "bg_secondary_acrylic",
        dark_acrylic: "bg_secondary_dark_acrylic",
      },
      bgTertiary: {
        light: "bg_tertiary_color",
        dark: "bg_tertiary_dark",
        light_acrylic: "bg_tertiary_acrylic",
        dark_acrylic: "bg_tertiary_dark_acrylic",
      },
      primary: {
        light: "primary_color",
        dark: "primary_dark",
        light_acrylic: "primary_acrylic",
        dark_acrylic: "primary_dark_acrylic",
      },
      secondary: {
        light: "secondary_color",
        dark: "secondary_dark",
        light_acrylic: "secondary_acrylic",
        dark_acrylic: "secondary_dark_acrylic",
      },
      textPrimary: {
        light: "text_primary_color",
        dark: "text_primary_dark",
        light_acrylic: "text_primary_acrylic",
        dark_acrylic: "text_primary_dark_acrylic",
      },
      textSecondary: {
        light: "text_secondary_color",
        dark: "text_secondary_dark",
        light_acrylic: "text_secondary_acrylic",
        dark_acrylic: "text_secondary_dark_acrylic",
      },
      border: {
        light: "border_color",
        dark: "border_dark",
        light_acrylic: "border_acrylic",
        dark_acrylic: "border_dark_acrylic",
      },
    };

    // 更新所有颜色方案的颜色值
    if (presetThemes[preset]) {
      Object.keys(colorTypes).forEach((colorType) => {
        colorPlans.forEach((plan) => {
          const settingsKey = colorTypes[colorType][plan];
          if (settingsKey && settings.value[settingsKey] !== undefined) {
            const presetColors = presetThemes[preset][plan];
            if (presetColors && presetColors[colorType]) {
              settings.value[settingsKey] = presetColors[colorType];
            }
          }
        });
      });
    }
  }

  if (settings.value.acrylic_enabled && acrylicSupported.value) {
    try {
      const isDark = effectiveTheme === "dark";
      await applyAcrylic(true, isDark);
    } catch {}
  }
  // 应用颜色变化
  applyColors();

  // 在颜色值更新后再保存
  markChanged();
}

async function saveSettings() {
  if (!settings.value) return;

  settings.value.default_max_memory = parseInt(maxMem.value) || 2048;
  settings.value.default_min_memory = parseInt(minMem.value) || 512;
  settings.value.default_port = parseInt(port.value) || 25565;
  settings.value.console_font_size = parseInt(fontSize.value) || 13;
  settings.value.max_log_lines = parseInt(logLines.value) || 5000;
  settings.value.background_opacity = parseFloat(bgOpacity.value) || 0.3;
  settings.value.background_blur = parseInt(bgBlur.value) || 0;
  settings.value.background_brightness = parseFloat(bgBrightness.value) || 1.0;
  settings.value.font_size = parseInt(uiFontSize.value) || 14;
  settings.value.color = settings.value.color || "default";

  saving.value = true;
  error.value = null;
  try {
    await settingsApi.save(settings.value);

    localStorage.setItem(
      "sl_theme_cache",
      JSON.stringify({
        theme: settings.value.theme || "auto",
        fontSize: settings.value.font_size || 14,
      }),
    );

    applyTheme(settings.value.theme);
    applyFontSize(settings.value.font_size);

    if (acrylicSupported.value) {
      try {
        const isDark = getEffectiveTheme(settings.value.theme) === "dark";
        await applyAcrylic(settings.value.acrylic_enabled, isDark);
      } catch {}
    }

    window.dispatchEvent(new CustomEvent("settings-updated"));
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function resetSettings() {
  try {
    const s = await settingsApi.reset();
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
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

async function exportSettings() {
  try {
    const json = await settingsApi.exportJson();
    await navigator.clipboard.writeText(json);
  } catch (e) {
    error.value = String(e);
  }
}

async function handleImport() {
  if (!importJson.value.trim()) {
    error.value = i18n.t("settings.no_json");
    return;
  }
  try {
    const s = await settingsApi.importJson(importJson.value);
    settings.value = s;
    maxMem.value = String(s.default_max_memory);
    minMem.value = String(s.default_min_memory);
    port.value = String(s.default_port);
    fontSize.value = String(s.console_font_size);
    logLines.value = String(s.max_log_lines);
    bgOpacity.value = String(s.background_opacity);
    bgBlur.value = String(s.background_blur);
    bgBrightness.value = String(s.background_brightness);
    uiFontSize.value = String(s.font_size);
    showImportModal.value = false;
    importJson.value = "";
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
    <div v-if="error" class="msg-banner error-banner">
      <span>{{ error }}</span>
      <button @click="error = null">x</button>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t("settings.loading") }}</span>
    </div>

    <template v-else-if="settings">
      <SLCard>
        <div
          style="
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: var(--sl-space-sm);
          "
        >
          <div>
            <h3
              style="
                margin: 0;
                font-size: 1.125rem;
                font-weight: 600;
                color: var(--sl-text-primary);
              "
            >
              {{ i18n.t("settings.color_theme") }}
            </h3>
            <p
              style="
                margin: var(--sl-space-xs) 0 0 0;
                font-size: 0.875rem;
                color: var(--sl-text-secondary);
              "
            >
              {{ i18n.t("settings.color_theme_desc") }}
            </p>
          </div>
          <div class="input-lg">
            <SLSelect
              v-model="settings.color"
              :options="colorOptions"
              @update:modelValue="handleThemeChange"
            />
          </div>
        </div>

        <!-- 颜色编辑折叠区域 -->
        <Transition name="color-section">
          <div class="collapsible-section" v-if="settings.color === 'custom'">
            <div class="collapsible-header" @click="colorSettingsExpanded = !colorSettingsExpanded">
              <div class="setting-info">
                <span class="setting-label">{{ i18n.t("settings.color_editing") }}</span>
                <span class="setting-desc">{{ i18n.t("settings.color_editing_desc") }}</span>
              </div>
              <div class="collapsible-toggle" :class="{ expanded: colorSettingsExpanded }">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              </div>
            </div>
            <Transition name="collapse">
              <div v-show="colorSettingsExpanded" class="collapsible-content">
                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.color_plan") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.color_plan_desc") }}</span>
                  </div>
                  <div class="input-lg">
                    <SLSelect
                      v-model="editColorPlan"
                      :options="editColorOptions"
                      @update:modelValue="applyColors"
                    />
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{
                      i18n.t("settings.primary_background_color")
                    }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="bgColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: bgColor }"
                      @click="showColorPicker('bgColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{
                      i18n.t("settings.secondary_background_color")
                    }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="bgSecondaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: bgSecondaryColor }"
                      @click="showColorPicker('bgSecondaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{
                      i18n.t("settings.tertiary_background_color")
                    }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="bgTertiaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: bgTertiaryColor }"
                      @click="showColorPicker('bgTertiaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.primary_color") }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="primaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: primaryColor }"
                      @click="showColorPicker('primaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.secondary_color") }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="secondaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: secondaryColor }"
                      @click="showColorPicker('secondaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.primary_text_color") }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="textPrimaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{
                        backgroundColor: textPrimaryColor,
                        color: textPrimaryColor === '#ffffff' ? '#000000' : '#ffffff',
                      }"
                      @click="showColorPicker('textPrimaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.secondary_text_color") }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="textSecondaryColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{
                        backgroundColor: textSecondaryColor,
                        color: textSecondaryColor === '#ffffff' ? '#000000' : '#ffffff',
                      }"
                      @click="showColorPicker('textSecondaryColor')"
                    ></div>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.border_color") }}</span>
                  </div>
                  <div class="input-lg color-input-container">
                    <SLInput
                      v-model="borderColor"
                      type="text"
                      :placeholder="i18n.t('settings.color_value')"
                    />
                    <div
                      class="color-preview"
                      :style="{ backgroundColor: borderColor }"
                      @click="showColorPicker('borderColor')"
                    ></div>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </Transition>
      </SLCard>

      <!-- Appearance -->
      <SLCard :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.theme") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.theme_desc") }}</span>
            </div>
            <div class="input-lg">
              <SLSelect
                v-model="settings.theme"
                :options="themeOptions"
                @update:modelValue="handleThemeChange"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.font_size") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.font_size_desc") }}</span>
            </div>
            <div class="slider-control">
              <input
                type="range"
                min="12"
                max="24"
                step="1"
                v-model="uiFontSize"
                @input="handleFontSizeChange"
                class="sl-slider"
              />
              <span class="slider-value">{{ uiFontSize }}px</span>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.font_family") }}</span>
              <span class="setting-desc">{{ i18n.t("settings.font_family_desc") }}</span>
            </div>
            <div class="input-lg">
              <SLSelect
                v-model="settings.font_family"
                :options="fontFamilyOptions"
                :searchable="true"
                :loading="fontsLoading"
                :previewFont="true"
                :placeholder="i18n.t('settings.search_font')"
                @update:modelValue="handleFontFamilyChange"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t("settings.acrylic") }}</span>
              <span class="setting-desc">
                {{
                  acrylicSupported
                    ? i18n.t("settings.acrylic_desc")
                    : i18n.t("settings.acrylic_not_supported")
                }}
              </span>
            </div>
            <SLSwitch
              v-model="settings.acrylic_enabled"
              :disabled="!acrylicSupported"
              @update:modelValue="handleAcrylicChange"
            />
          </div>

          <!-- 背景图片折叠区域 -->
          <div class="collapsible-section">
            <div class="collapsible-header" @click="bgSettingsExpanded = !bgSettingsExpanded">
              <div class="setting-info">
                <span class="setting-label">{{ i18n.t("settings.background") }}</span>
                <span class="setting-desc">{{ i18n.t("settings.background_desc") }}</span>
              </div>
              <div class="collapsible-toggle" :class="{ expanded: bgSettingsExpanded }">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              </div>
            </div>
            <Transition name="collapse">
              <div v-show="bgSettingsExpanded" class="collapsible-content">
                <div class="setting-row full-width">
                  <div class="bg-image-picker">
                    <div v-if="settings.background_image" class="bg-preview">
                      <div v-if="bgPreviewLoading && !bgPreviewLoaded" class="bg-preview-loading">
                        <div class="loading-spinner"></div>
                        <span>{{ i18n.t("settings.loading_preview") }}</span>
                      </div>
                      <img
                        v-show="bgPreviewLoaded || !bgPreviewLoading"
                        :src="backgroundPreviewUrl"
                        alt="Background preview"
                        @load="
                          bgPreviewLoaded = true;
                          bgPreviewLoading = false;
                        "
                        @loadstart="bgPreviewLoading = true"
                        @error="bgPreviewLoading = false"
                        loading="lazy"
                      />
                      <div
                        v-if="isAnimatedImage(settings.background_image)"
                        class="bg-animated-badge"
                      >
                        {{ i18n.t("settings.animated_image") }}
                      </div>
                      <div class="bg-preview-overlay">
                        <span class="bg-preview-path">{{
                          settings.background_image.split("\\").pop()
                        }}</span>
                        <SLButton variant="danger" size="sm" @click="clearBackgroundImage">{{
                          i18n.t("settings.remove")
                        }}</SLButton>
                      </div>
                    </div>
                    <SLButton v-else variant="secondary" @click="pickBackgroundImage">
                      {{ i18n.t("settings.pick_image") }}
                    </SLButton>
                    <SLButton
                      v-if="settings.background_image"
                      variant="secondary"
                      size="sm"
                      @click="pickBackgroundImage"
                    >
                      {{ i18n.t("settings.replace_image") }}
                    </SLButton>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.opacity") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.opacity_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.05"
                      v-model="bgOpacity"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgOpacity }}</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.blur") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.blur_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="20"
                      step="1"
                      v-model="bgBlur"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgBlur }}px</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.brightness") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.brightness_desc") }}</span>
                  </div>
                  <div class="slider-control">
                    <input
                      type="range"
                      min="0"
                      max="2"
                      step="0.1"
                      v-model="bgBrightness"
                      @input="markChanged"
                      class="sl-slider"
                    />
                    <span class="slider-value">{{ bgBrightness }}</span>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t("settings.background_size") }}</span>
                    <span class="setting-desc">{{ i18n.t("settings.background_size_desc") }}</span>
                  </div>
                  <div class="input-lg">
                    <SLSelect
                      v-model="settings.background_size"
                      :options="backgroundSizeOptions"
                      @update:modelValue="markChanged"
                    />
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </SLCard>

      <!-- Actions -->
      <div class="settings-actions">
        <div class="actions-left"></div>
        <div class="actions-right">{{ i18n.t("settings.personalize_page_import_export") }}</div>
      </div>
    </template>

    <SLModal
      :visible="showImportModal"
      :title="i18n.t('settings.import_settings')"
      @close="showImportModal = false"
    >
      <div class="import-form">
        <p class="text-caption">{{ i18n.t("settings.import_desc") }}</p>
        <textarea
          class="import-textarea"
          v-model="importJson"
          placeholder='{"close_servers_on_exit": true, ...}'
          rows="10"
        ></textarea>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showImportModal = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="primary" @click="handleImport">{{
          i18n.t("settings.confirm_import")
        }}</SLButton>
      </template>
    </SLModal>

    <SLModal
      :visible="showResetConfirm"
      :title="i18n.t('settings.reset_confirm')"
      @close="showResetConfirm = false"
    >
      <p class="text-body">{{ i18n.t("settings.reset_desc") }}</p>
      <template #footer>
        <SLButton variant="secondary" @click="showResetConfirm = false">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="danger" @click="resetSettings">{{
          i18n.t("settings.confirm_reset")
        }}</SLButton>
      </template>
    </SLModal>

    <!-- 颜色选择器对话框 -->
    <SLModal
      :visible="showColorPickerDialog"
      :title="i18n.t('settings.color_picker')"
      @close="closeColorPicker"
      :width="320"
    >
      <div class="color-picker-content">
        <!-- 颜色预览 -->
        <div class="color-preview-container">
          <div class="color-preview-background"></div>
          <div class="color-preview-large" :style="{ backgroundColor: currentColorValue }"></div>
        </div>

        <!-- 颜色值输入 -->
        <div class="color-value-input">
          <SLInput
            v-model="currentColorValue"
            type="text"
            :placeholder="i18n.t('settings.input_color_placeholder')"
            @input="updateColor(currentColorValue)"
          />
        </div>

        <!-- 色相滑动条 -->
        <div class="picker-section">
          <label>{{ i18n.t("settings.color_hue") }}</label>
          <div class="hue-slider" @click="updateHueFromClick($event)">
            <div class="hue-track"></div>
            <div
              class="hue-thumb"
              :style="{ left: (hsl.h / 360) * 100 + '%' }"
              @mousedown="handleHueDrag($event)"
            ></div>
          </div>
        </div>

        <!-- 饱和度滑动条 -->
        <div class="picker-section">
          <label>{{ i18n.t("settings.color_saturation") }}</label>
          <div class="saturation-slider" @click="updateSaturationFromClick($event)">
            <div
              class="saturation-track"
              :style="{
                background: `linear-gradient(to right, hsl(${hsl.h}, 0%, 50%), hsl(${hsl.h}, 100%, 50%))`,
              }"
            ></div>
            <div
              class="saturation-thumb"
              :style="{ left: hsl.s + '%' }"
              @mousedown="handleSaturationDrag($event)"
            ></div>
          </div>
        </div>

        <!-- 亮度滑动条 -->
        <div class="picker-section">
          <label>{{ i18n.t("settings.color_lightness") }}</label>
          <div class="lightness-slider" @click="updateLightnessFromClick($event)">
            <div
              class="lightness-track"
              :style="{
                background: `linear-gradient(to right, hsl(${hsl.h}, ${hsl.s}%, 0%), hsl(${hsl.h}, ${hsl.s}%, 50%), hsl(${hsl.h}, ${hsl.s}%, 100%))`,
              }"
            ></div>
            <div
              class="lightness-thumb"
              :style="{ left: hsl.l + '%' }"
              @mousedown="handleLightnessDrag($event)"
            ></div>
          </div>
        </div>

        <!-- 透明度滑动条 -->
        <div class="picker-section">
          <label>{{ i18n.t("settings.color_alpha") }}</label>
          <div class="alpha-slider-container">
            <div class="alpha-slider-background"></div>
            <div class="alpha-slider" @click="updateAlphaFromClick($event)">
              <div
                class="alpha-track"
                :style="{
                  background: `linear-gradient(to right, transparent, hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%))`,
                }"
              ></div>
              <div
                class="alpha-thumb"
                :style="{ left: rgb.a * 100 + '%' }"
                @mousedown="handleAlphaDrag($event)"
              ></div>
            </div>
          </div>
        </div>

        <!-- RGB输入 -->
        <div class="rgb-inputs">
          <div class="rgb-input-item">
            <label>R</label>
            <SLInput
              type="number"
              min="0"
              max="255"
              v-model="rgb.r"
              @input="updateFromRGB"
              style="width: 100px"
            />
          </div>
          <div class="rgb-input-item">
            <label>G</label>
            <SLInput
              type="number"
              min="0"
              max="255"
              v-model="rgb.g"
              @input="updateFromRGB"
              style="width: 100px"
            />
          </div>
          <div class="rgb-input-item">
            <label>B</label>
            <SLInput
              type="number"
              min="0"
              max="255"
              v-model="rgb.b"
              @input="updateFromRGB"
              style="width: 100px"
            />
          </div>
          <div class="rgb-input-item">
            <label>A</label>
            <SLInput
              type="number"
              min="0"
              max="1"
              step="0.01"
              v-model="rgb.a"
              @input="updateFromRGB"
              style="width: 100px"
            />
          </div>
        </div>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="cancelColorPicker">{{
          i18n.t("settings.cancel")
        }}</SLButton>
        <SLButton variant="primary" @click="confirmColorPicker">{{
          i18n.t("settings.confirm")
        }}</SLButton>
      </template>
    </SLModal>
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

.msg-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}
.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--sl-error);
}
.msg-banner button {
  font-weight: 600;
  color: inherit;
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

.settings-group {
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-bottom: 1px solid var(--sl-border-light);
  gap: var(--sl-space-lg);
}
.setting-row:last-child {
  border-bottom: none;
}
.setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.setting-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}
.setting-desc {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  line-height: 1.4;
}

.input-sm {
  width: 120px;
  flex-shrink: 0;
}
.input-lg {
  width: 320px;
  flex-shrink: 0;
}

.color-input-container {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-preview {
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  flex-shrink: 0;
}

.jvm-textarea,
.import-textarea {
  width: 100%;
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  resize: vertical;
  line-height: 1.6;
}
.jvm-textarea:focus,
.import-textarea:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px var(--sl-primary-bg);
  outline: none;
}

.settings-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-top: 1px solid var(--sl-border);
}
.actions-left,
.actions-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.bg-image-picker {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.bg-preview {
  position: relative;
  width: 100%;
  max-width: 400px;
  height: 200px;
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  border: 1px solid var(--sl-border);
}

.bg-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.bg-preview-loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  background: var(--sl-surface);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.bg-animated-badge {
  position: absolute;
  top: var(--sl-space-sm);
  right: var(--sl-space-sm);
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 0.75rem;
  font-weight: 500;
  border-radius: var(--sl-radius-sm);
}

.bg-preview-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.bg-preview-path {
  font-size: 0.8125rem;
  color: white;
  font-family: var(--sl-font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 颜色编辑区域过渡动画 */
.color-section-enter-active,
.color-section-leave-active {
  transition: all 0.3s ease;
  max-height: 800px;
  overflow: hidden;
}

.color-section-enter-from,
.color-section-leave-to {
  opacity: 0;
  max-height: 0;
  overflow: hidden;
  margin: 0;
  padding: 0;
}

.slider-control {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  min-width: 200px;
}

.sl-slider {
  flex: 1;
  height: 6px;
  border-radius: var(--sl-radius-full);
  background: var(--sl-border);
  outline: none;
  -webkit-appearance: none;
}

.sl-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.sl-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  cursor: pointer;
  border: none;
  transition: all var(--sl-transition-fast);
}

.sl-slider::-moz-range-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.slider-value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  min-width: 50px;
  text-align: right;
}

.collapsible-section {
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  margin: var(--sl-space-sm) 0;
}

.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md);
  cursor: pointer;
  background: var(--sl-surface);
  transition: background-color var(--sl-transition-fast);
}

.collapsible-header:hover {
  background: var(--sl-surface-hover);
}

.collapsible-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-normal);
  flex-shrink: 0;
}

.collapsible-toggle:hover {
  background: var(--sl-border-light);
  color: var(--sl-text-primary);
}

.collapsible-toggle.expanded {
  transform: rotate(180deg);
}

.collapsible-content {
  padding: 0 var(--sl-space-md) var(--sl-space-md);
  background: var(--sl-surface);
}

.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 800px;
}

/* 颜色选择器样式 */
.color-picker-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  padding: var(--sl-space-sm);
}

.color-preview-container {
  position: relative;
  width: 100%;
  height: 60px;
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  border: 1px solid var(--sl-border);
}

.color-preview-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: linear-gradient(
    45deg,
    #f0f0f0 25%,
    #ffffff 25%,
    #ffffff 50%,
    #f0f0f0 50%,
    #f0f0f0 75%,
    #ffffff 75%,
    #ffffff 100%
  );
  background-size: 16px 16px;
}

.color-preview-large {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.color-value-input {
  width: 100%;
}

.picker-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.picker-section label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.hue-slider {
  position: relative;
  width: 100%;
  height: 6px;
  border-radius: var(--sl-radius-full);
  cursor: pointer;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.hue-track {
  width: 100%;
  height: 100%;
  border-radius: var(--sl-radius-full);
  background: linear-gradient(
    to right,
    #ff0000,
    #ffff00,
    #00ff00,
    #00ffff,
    #0000ff,
    #ff00ff,
    #ff0000
  );
}

.hue-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  border: 2px solid white;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  z-index: 10;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.hue-thumb:hover {
  transform: translate(-50%, -50%) scale(1.15);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.saturation-slider {
  position: relative;
  width: 100%;
  height: 6px;
  border-radius: var(--sl-radius-full);
  cursor: pointer;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.saturation-track {
  width: 100%;
  height: 100%;
  border-radius: var(--sl-radius-full);
  background: linear-gradient(to right, #808080, #ff0000);
}

.saturation-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  border: 2px solid white;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  z-index: 10;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.saturation-thumb:hover {
  transform: translate(-50%, -50%) scale(1.15);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.lightness-slider {
  position: relative;
  width: 100%;
  height: 6px;
  border-radius: var(--sl-radius-full);
  cursor: pointer;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.lightness-track {
  width: 100%;
  height: 100%;
  border-radius: var(--sl-radius-full);
  background: linear-gradient(to right, #000000, #ff0000, #ffffff);
}

.lightness-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  border: 2px solid white;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  z-index: 10;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.lightness-thumb:hover {
  transform: translate(-50%, -50%) scale(1.15);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.alpha-slider-container {
  position: relative;
  width: 100%;
  height: 6px;
  border-radius: var(--sl-radius-full);
  overflow: visible;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.1);
}

.alpha-slider-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: linear-gradient(
    45deg,
    #f0f0f0 25%,
    #ffffff 25%,
    #ffffff 50%,
    #f0f0f0 50%,
    #f0f0f0 75%,
    #ffffff 75%,
    #ffffff 100%
  );
  background-size: 8px 8px;
  border-radius: var(--sl-radius-full);
  z-index: 1;
}

.alpha-slider {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border-radius: var(--sl-radius-full);
  cursor: pointer;
  z-index: 2;
}

.alpha-track {
  width: 100%;
  height: 100%;
  border-radius: var(--sl-radius-full);
}

.alpha-thumb {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--sl-primary);
  border: 2px solid white;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  z-index: 10;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.alpha-thumb:hover {
  transform: translate(-50%, -50%) scale(1.15);
  box-shadow: 0 0 0 4px var(--sl-primary-bg);
}

.rgb-inputs {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--sl-space-sm);
}

.rgb-input-item {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.rgb-input-item label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.rgb-input-item .sl-input {
  width: 100%;
}
</style>
