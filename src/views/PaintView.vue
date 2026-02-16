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

const settings = ref<AppSettings | null>(null);
const loading = ref(true);
const fontsLoading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);
const hasChanges = ref(false);

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

const backgroundSizeOptions = [
  { label: i18n.t('settings.background_size_options.cover'), value: "cover" },
  { label: i18n.t('settings.background_size_options.contain'), value: "contain" },
  { label: i18n.t('settings.background_size_options.fill'), value: "fill" },
  { label: i18n.t('settings.background_size_options.auto'), value: "auto" },
];

const colorOptions = [
  { label: i18n.t('settings.color_options.default'), value: "default" },
  { label: i18n.t('settings.color_options.midnight'), value: "midnight" },
  { label: i18n.t('settings.color_options.forest'), value: "forest" },
  { label: i18n.t('settings.color_options.sunset'), value: "sunset" },
  { label: i18n.t('settings.color_options.ocean'), value: "ocean" },
  { label: i18n.t('settings.color_options.rose'), value: "rose" },
  { label: i18n.t('settings.color_options.custom'), value: "custom" },
];

const editColorOptions = [
  { label: i18n.t('settings.edit_colorplan_options.light'), value: "light" },
  { label: i18n.t('settings.edit_colorplan_options.dark'), value: "dark" },
  { label: i18n.t('settings.edit_colorplan_options.light_acrylic'), value: "light_acrylic" },
  { label: i18n.t('settings.edit_colorplan_options.dark_acrylic'), value: "dark_acrylic" },
];

const themeOptions = [
  { label: i18n.t('settings.theme_options.auto'), value: "auto" },
  { label: i18n.t('settings.theme_options.light'), value: "light" },
  { label: i18n.t('settings.theme_options.dark'), value: "dark" },
];

const fontFamilyOptions = ref<{ label: string; value: string }[]>([
  { label: i18n.t('settings.font_family_default'), value: "" },
]);

const showImportModal = ref(false);
const importJson = ref("");
const showResetConfirm = ref(false);
const bgSettingsExpanded = ref(false);
const colorSettingsExpanded = ref(false);
const bgPreviewLoaded = ref(false);
const bgPreviewLoading = ref(false);
const editColorPlan = ref("light");

// 颜色值计算属性，用于动态绑定不同主题的颜色
const bgColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#f8fafc";
          case "dark":
            return "#0f1117";
          case "light_acrylic":
            return "rgba(248, 250, 252, 0.7)";
          case "dark_acrylic":
            return "rgba(15, 17, 23, 0.7)";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "bg");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.bg_color,
      dark: settings.value.bg_dark,
      light_acrylic: settings.value.bg_acrylic,
      dark_acrylic: settings.value.bg_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#f8fafc";
          case "dark":
            return "#0f1117";
          case "light_acrylic":
            return "rgba(248, 250, 252, 0.7)";
          case "dark_acrylic":
            return "rgba(15, 17, 23, 0.7)";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "bg");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.bg_color = value;
        break;
      case "dark":
        settings.value.bg_dark = value;
        break;
      case "light_acrylic":
        settings.value.bg_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.bg_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const bgSecondaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#f1f5f9";
          case "dark":
            return "#1a1d28";
          case "light_acrylic":
            return "rgba(241, 245, 249, 0.6)";
          case "dark_acrylic":
            return "rgba(26, 29, 40, 0.6)";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "bgSecondary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.bg_secondary_color,
      dark: settings.value.bg_secondary_dark,
      light_acrylic: settings.value.bg_secondary_acrylic,
      dark_acrylic: settings.value.bg_secondary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#f1f5f9";
          case "dark":
            return "#1a1d28";
          case "light_acrylic":
            return "rgba(241, 245, 249, 0.6)";
          case "dark_acrylic":
            return "rgba(26, 29, 40, 0.6)";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "bgSecondary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.bg_secondary_color = value;
        break;
      case "dark":
        settings.value.bg_secondary_dark = value;
        break;
      case "light_acrylic":
        settings.value.bg_secondary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.bg_secondary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const bgTertiaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#e2e8f0";
          case "dark":
            return "#242836";
          case "light_acrylic":
            return "rgba(226, 232, 240, 0.5)";
          case "dark_acrylic":
            return "rgba(36, 40, 54, 0.5)";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "bgTertiary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.bg_tertiary_color,
      dark: settings.value.bg_tertiary_dark,
      light_acrylic: settings.value.bg_tertiary_acrylic,
      dark_acrylic: settings.value.bg_tertiary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#e2e8f0";
          case "dark":
            return "#242836";
          case "light_acrylic":
            return "rgba(226, 232, 240, 0.5)";
          case "dark_acrylic":
            return "rgba(36, 40, 54, 0.5)";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "bgTertiary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.bg_tertiary_color = value;
        break;
      case "dark":
        settings.value.bg_tertiary_dark = value;
        break;
      case "light_acrylic":
        settings.value.bg_tertiary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.bg_tertiary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const primaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#0ea5e9";
          case "dark":
            return "#60a5fa";
          case "light_acrylic":
            return "#0ea5e9";
          case "dark_acrylic":
            return "#60a5fa";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "primary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.primary_color,
      dark: settings.value.primary_dark,
      light_acrylic: settings.value.primary_acrylic,
      dark_acrylic: settings.value.primary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#0ea5e9";
          case "dark":
            return "#60a5fa";
          case "light_acrylic":
            return "#0ea5e9";
          case "dark_acrylic":
            return "#60a5fa";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "primary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.primary_color = value;
        break;
      case "dark":
        settings.value.primary_dark = value;
        break;
      case "light_acrylic":
        settings.value.primary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.primary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const secondaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#06b6d4";
          case "dark":
            return "#22d3ee";
          case "light_acrylic":
            return "#06b6d4";
          case "dark_acrylic":
            return "#22d3ee";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "secondary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.secondary_color,
      dark: settings.value.secondary_dark,
      light_acrylic: settings.value.secondary_acrylic,
      dark_acrylic: settings.value.secondary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#06b6d4";
          case "dark":
            return "#22d3ee";
          case "light_acrylic":
            return "#06b6d4";
          case "dark_acrylic":
            return "#22d3ee";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "secondary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.secondary_color = value;
        break;
      case "dark":
        settings.value.secondary_dark = value;
        break;
      case "light_acrylic":
        settings.value.secondary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.secondary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const textPrimaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#0f172a";
          case "dark":
            return "#e2e8f0";
          case "light_acrylic":
            return "#0f172a";
          case "dark_acrylic":
            return "#e2e8f0";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "textPrimary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.text_primary_color,
      dark: settings.value.text_primary_dark,
      light_acrylic: settings.value.text_primary_acrylic,
      dark_acrylic: settings.value.text_primary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#0f172a";
          case "dark":
            return "#e2e8f0";
          case "light_acrylic":
            return "#0f172a";
          case "dark_acrylic":
            return "#e2e8f0";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "textPrimary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.text_primary_color = value;
        break;
      case "dark":
        settings.value.text_primary_dark = value;
        break;
      case "light_acrylic":
        settings.value.text_primary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.text_primary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const textSecondaryColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#475569";
          case "dark":
            return "#94a3b8";
          case "light_acrylic":
            return "#475569";
          case "dark_acrylic":
            return "#94a3b8";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "textSecondary");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.text_secondary_color,
      dark: settings.value.text_secondary_dark,
      light_acrylic: settings.value.text_secondary_acrylic,
      dark_acrylic: settings.value.text_secondary_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#475569";
          case "dark":
            return "#94a3b8";
          case "light_acrylic":
            return "#475569";
          case "dark_acrylic":
            return "#94a3b8";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "textSecondary");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.text_secondary_color = value;
        break;
      case "dark":
        settings.value.text_secondary_dark = value;
        break;
      case "light_acrylic":
        settings.value.text_secondary_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.text_secondary_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const borderColor = computed({
  get: () => {
    if (!settings.value) return "";
    // 如果不是自定义主题，返回预设颜色值
    if (settings.value.color !== "custom") {
      // 如果是默认主题，返回默认颜色值
      if (settings.value.color === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#e2e8f0";
          case "dark":
            return "rgba(255, 255, 255, 0.1)";
          case "light_acrylic":
            return "#e2e8f0";
          case "dark_acrylic":
            return "rgba(255, 255, 255, 0.1)";
          default:
            return "";
        }
      }
      // 其他预设主题，返回对应的预设颜色值
      return getPresetColor(settings.value.color, editColorPlan.value, "border");
    }
    // 自定义主题，返回设置中的颜色值，如果没有则使用当前预设的颜色值
    const customColor = {
      light: settings.value.border_color,
      dark: settings.value.border_dark,
      light_acrylic: settings.value.border_acrylic,
      dark_acrylic: settings.value.border_dark_acrylic,
    }[editColorPlan.value];

    if (customColor) return customColor;

    // 如果没有自定义颜色，使用当前预设的颜色值
    if (settings.value.color_prev) {
      if (settings.value.color_prev === "default") {
        switch (editColorPlan.value) {
          case "light":
            return "#e2e8f0";
          case "dark":
            return "rgba(255, 255, 255, 0.1)";
          case "light_acrylic":
            return "#e2e8f0";
          case "dark_acrylic":
            return "rgba(255, 255, 255, 0.1)";
          default:
            return "";
        }
      }
      return getPresetColor(settings.value.color_prev, editColorPlan.value, "border");
    }

    // 默认返回空
    return "";
  },
  set: (value) => {
    if (!settings.value) return;
    switch (editColorPlan.value) {
      case "light":
        settings.value.border_color = value;
        break;
      case "dark":
        settings.value.border_dark = value;
        break;
      case "light_acrylic":
        settings.value.border_acrylic = value;
        break;
      case "dark_acrylic":
        settings.value.border_dark_acrylic = value;
        break;
    }
    // 自动切换到自定义预设
    if (settings.value.color !== "custom") {
      settings.value.color = "custom";
    }
    markChanged();
    // 应用颜色变化
    applyColors();
  },
});

const backgroundPreviewUrl = computed(() => {
  if (!settings.value?.background_image) return "";
  if (!bgSettingsExpanded.value) return "";
  return convertFileSrc(settings.value.background_image);
});

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
      { label: i18n.t('settings.font_family_default'), value: "" },
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

// 监控颜色值变化，自动切换到自定义预设
watch(
  () => [
    settings.value?.bg_color,
    settings.value?.bg_secondary_color,
    settings.value?.bg_tertiary_color,
    settings.value?.primary_color,
    settings.value?.secondary_color,
    settings.value?.text_primary_color,
    settings.value?.text_secondary_color,
    settings.value?.border_color,
    settings.value?.bg_dark,
    settings.value?.bg_secondary_dark,
    settings.value?.bg_tertiary_dark,
    settings.value?.primary_dark,
    settings.value?.secondary_dark,
    settings.value?.text_primary_dark,
    settings.value?.text_secondary_dark,
    settings.value?.border_dark,
    settings.value?.bg_acrylic,
    settings.value?.bg_secondary_acrylic,
    settings.value?.bg_tertiary_acrylic,
    settings.value?.primary_acrylic,
    settings.value?.secondary_acrylic,
    settings.value?.text_primary_acrylic,
    settings.value?.text_secondary_acrylic,
    settings.value?.border_acrylic,
    settings.value?.bg_dark_acrylic,
    settings.value?.bg_secondary_dark_acrylic,
    settings.value?.bg_tertiary_dark_acrylic,
    settings.value?.primary_dark_acrylic,
    settings.value?.secondary_dark_acrylic,
    settings.value?.text_primary_dark_acrylic,
    settings.value?.text_secondary_dark_acrylic,
    settings.value?.border_dark_acrylic,
  ],
  (newValues, oldValues) => {
    if (!settings.value) return;

    // 检查是否是初始化（oldValues 都是 undefined）
    const isInitialization = oldValues.every((val) => val === undefined);
    if (isInitialization) return;

    // 检查是否有任何颜色值发生了变化
    const hasColorChanged = newValues.some((newVal, index) => {
      return newVal !== oldValues[index];
    });

    // 如果颜色值发生了变化，且当前不是自定义预设，则切换到自定义
    if (hasColorChanged && settings.value.color !== "custom") {
      // 保存当前主题作为之前的主题
      settings.value.color_prev = settings.value.color;
      settings.value.color = "custom";
      markChanged();
    }
  },
  { deep: true },
);

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
    hasChanges.value = false;
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
  hasChanges.value = true;
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
function getPresetColor(preset, plan, colorType) {
  if (!presetThemes[preset]) return "";

  let themeKey;
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

  return presetThemes[preset][themeKey][colorType] || "";
}

// 监听颜色主题变化，保存之前的主题
watch(
  () => settings.value?.color,
  (newColor, oldColor) => {
    if (!settings.value) return;

    // 保存之前的主题，只有当旧主题不是自定义时才保存
    if (oldColor && oldColor !== "custom") {
      settings.value.color_prev = oldColor;
    }

    // 应用颜色变化
    applyColors();
  },
);

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

  // 应用颜色变化
  applyColors();
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
    success.value = i18n.t('settings.saved');
    hasChanges.value = false;
    setTimeout(() => (success.value = null), 3000);

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
    hasChanges.value = false;
    settings.value.color = "default";
    success.value = i18n.t('settings.reset_success');
    setTimeout(() => (success.value = null), 3000);
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
    success.value = i18n.t('settings.export_success');
    setTimeout(() => (success.value = null), 3000);
  } catch (e) {
    error.value = String(e);
  }
}

async function handleImport() {
  if (!importJson.value.trim()) {
    error.value = i18n.t('settings.no_json');
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
    hasChanges.value = false;
    success.value = i18n.t('settings.import_success');
    setTimeout(() => (success.value = null), 3000);
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
    <div v-if="success" class="msg-banner success-banner">
      <span>{{ success }}</span>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>{{ i18n.t('settings.loading') }}</span>
    </div>

    <template v-else-if="settings">
      <SLCard :title="i18n.t('settings.color_theme')" :subtitle="i18n.t('settings.color_theme_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t('settings.use_preset_colortheme') }}</span>
              <span class="setting-desc">{{ i18n.t('settings.use_preset_colortheme_desc') }}</span>
            </div>
            <div class="input-lg">
              <SLSelect
                v-model="settings.color"
                :options="colorOptions"
                @update:modelValue="handleThemeChange"
              />
            </div>
          </div>
        </div>
        <div class="collapsible-section">
          <div class="collapsible-header" @click="colorSettingsExpanded = !colorSettingsExpanded">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t('settings.color_editing') }}</span>
              <span class="setting-desc">{{ i18n.t('settings.color_editing_desc') }}</span>
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
                  <span class="setting-label">{{ i18n.t('settings.current_edit_colorplan') }}</span>
                  <span class="setting-desc">{{ i18n.t('settings.current_edit_colorplan_desc') }}</span>
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
                  <span class="setting-label">{{ i18n.t('settings.primary_background_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="bgColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div class="color-preview" :style="{ backgroundColor: bgColor }"></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.secondary_background_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="bgSecondaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div class="color-preview" :style="{ backgroundColor: bgSecondaryColor }"></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.tertiary_background_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="bgTertiaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div class="color-preview" :style="{ backgroundColor: bgTertiaryColor }"></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.primary_emphasis_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="primaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div class="color-preview" :style="{ backgroundColor: primaryColor }"></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.secondary_emphasis_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="secondaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div class="color-preview" :style="{ backgroundColor: secondaryColor }"></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.text_primary_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="textPrimaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div
                    class="color-preview"
                    :style="{ backgroundColor: textPrimaryColor, border: '1px solid #e2e8f0' }"
                  ></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.text_secondary_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="textSecondaryColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div
                    class="color-preview"
                    :style="{ backgroundColor: textSecondaryColor, border: '1px solid #e2e8f0' }"
                  ></div>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">{{ i18n.t('settings.border_color') }}</span>
                </div>
                <div class="input-lg color-input-container">
                  <SLInput v-model="borderColor" type="text" :placeholder="i18n.t('settings.color_value')" />
                  <div
                    class="color-preview"
                    :style="{ backgroundColor: borderColor, border: '1px solid #e2e8f0' }"
                  ></div>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </SLCard>
      <!-- Appearance -->
      <SLCard :title="i18n.t('settings.appearance')" :subtitle="i18n.t('settings.appearance_desc')">
        <div class="settings-group">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">{{ i18n.t('settings.theme') }}</span>
              <span class="setting-desc"
                >{{ i18n.t('settings.theme_desc') }}</span
              >
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
              <span class="setting-label">{{ i18n.t('settings.font_size') }}</span>
              <span class="setting-desc">{{ i18n.t('settings.font_size_desc') }}</span>
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
              <span class="setting-label">{{ i18n.t('settings.font_family') }}</span>
              <span class="setting-desc"
                >{{ i18n.t('settings.font_family_desc') }}</span
              >
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
              <span class="setting-label">{{ i18n.t('settings.acrylic') }}</span>
              <span class="setting-desc">
                {{
                  acrylicSupported
                    ? i18n.t('settings.acrylic_desc')
                    : i18n.t('settings.acrylic_not_supported')
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
                <span class="setting-label">{{ i18n.t('settings.background') }}</span>
                <span class="setting-desc"
                  >{{ i18n.t('settings.background_desc') }}</span
                >
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
                        <span>{{ i18n.t('settings.loading_preview') }}</span>
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
                        {{ i18n.t('settings.animated_image') }}
                      </div>
                      <div class="bg-preview-overlay">
                        <span class="bg-preview-path">{{
                          settings.background_image.split("\\").pop()
                        }}</span>
                        <SLButton variant="danger" size="sm" @click="clearBackgroundImage"
                          >{{ i18n.t('settings.remove') }}</SLButton
                        >
                      </div>
                    </div>
                    <SLButton v-else variant="secondary" @click="pickBackgroundImage">
                      {{ i18n.t('settings.pick_image') }}
                    </SLButton>
                    <SLButton
                      v-if="settings.background_image"
                      variant="secondary"
                      size="sm"
                      @click="pickBackgroundImage"
                    >
                      {{ i18n.t('settings.replace_image') }}
                    </SLButton>
                  </div>
                </div>

                <div class="setting-row">
                  <div class="setting-info">
                    <span class="setting-label">{{ i18n.t('settings.opacity') }}</span>
                    <span class="setting-desc"
                      >{{ i18n.t('settings.opacity_desc') }}</span
                    >
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
                    <span class="setting-label">{{ i18n.t('settings.blur') }}</span>
                    <span class="setting-desc">{{ i18n.t('settings.blur_desc') }}</span>
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
                    <span class="setting-label">{{ i18n.t('settings.brightness') }}</span>
                    <span class="setting-desc">{{ i18n.t('settings.brightness_desc') }}</span>
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
                    <span class="setting-label">{{ i18n.t('settings.background_size') }}</span>
                    <span class="setting-desc">{{ i18n.t('settings.background_size_desc') }}</span>
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
        <div class="actions-left">
          <SLButton variant="primary" size="lg" :loading="saving" @click="saveSettings">
            {{ i18n.t('settings.save') }}
          </SLButton>
          <SLButton variant="secondary" @click="loadSettings">{{ i18n.t('settings.discard') }}</SLButton>
          <span v-if="hasChanges" class="unsaved-hint">{{ i18n.t('settings.unsaved_changes') }}</span>
        </div>
        <div class="actions-right">{{ i18n.t('settings.personalize_page_import_export') }}</div>
      </div>
    </template>

    <SLModal :visible="showImportModal" :title="i18n.t('settings.import_settings')" @close="showImportModal = false">
      <div class="import-form">
        <p class="text-caption">{{ i18n.t('settings.import_desc') }}</p>
        <textarea
          class="import-textarea"
          v-model="importJson"
          placeholder='{"close_servers_on_exit": true, ...}'
          rows="10"
        ></textarea>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showImportModal = false">{{ i18n.t('settings.cancel') }}</SLButton>
        <SLButton variant="primary" @click="handleImport">{{ i18n.t('settings.confirm_import') }}</SLButton>
      </template>
    </SLModal>

    <SLModal :visible="showResetConfirm" :title="i18n.t('settings.reset_confirm')" @close="showResetConfirm = false">
      <p class="text-body">{{ i18n.t('settings.reset_desc') }}</p>
      <template #footer>
        <SLButton variant="secondary" @click="showResetConfirm = false">{{ i18n.t('settings.cancel') }}</SLButton>
        <SLButton variant="danger" @click="resetSettings">{{ i18n.t('settings.confirm_reset') }}</SLButton>
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
.success-banner {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: var(--sl-success);
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

.unsaved-hint {
  font-size: 0.8125rem;
  color: var(--sl-warning);
  font-weight: 500;
  padding: 2px 10px;
  background: rgba(245, 158, 11, 0.1);
  border-radius: var(--sl-radius-full);
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
</style>
