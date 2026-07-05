import { ref } from "vue";
import type { AppSettings } from "@api/settings";

export function usePaintSettingsFields() {
  const fontSize = ref("14");
  const memoryDisplayPrecision = ref(2);
  const consoleFontSize = ref("13");
  const consoleFontFamily = ref("");
  const consoleLetterSpacing = ref("0");
  const maxLogLines = ref("5000");
  const bgOpacity = ref("0.3");
  const bgBlur = ref("0");
  const bgBrightness = ref("1.0");

  function syncLocalValues(settings: AppSettings) {
    fontSize.value = String(settings.font_size);
    memoryDisplayPrecision.value = settings.memory_display_precision ?? 2;
    consoleFontSize.value = String(settings.console_font_size);
    consoleFontFamily.value = settings.console_font_family || "";
    consoleLetterSpacing.value = String(settings.console_letter_spacing ?? 0);
    maxLogLines.value = String(settings.max_log_lines);
    bgOpacity.value = String(settings.background_opacity);
    bgBlur.value = String(settings.background_blur);
    bgBrightness.value = String(settings.background_brightness);
  }

  function prepareForSave(settings: AppSettings) {
    settings.console_font_size = parseInt(consoleFontSize.value, 10) || 13;
    settings.console_font_family = consoleFontFamily.value;
    settings.console_letter_spacing = parseInt(consoleLetterSpacing.value, 10) || 0;
    settings.max_log_lines = parseInt(maxLogLines.value, 10) || 5000;
    settings.background_opacity = parseFloat(bgOpacity.value) || 0.3;
    settings.background_blur = parseInt(bgBlur.value, 10) || 0;
    settings.background_brightness = parseFloat(bgBrightness.value) || 1.0;
    settings.font_size = parseInt(fontSize.value, 10) || 14;
    settings.memory_display_precision = [0, 2, 4].includes(memoryDisplayPrecision.value)
      ? memoryDisplayPrecision.value
      : 2;
    settings.color = settings.color || "default";
  }

  return {
    fontSize,
    memoryDisplayPrecision,
    consoleFontSize,
    consoleFontFamily,
    consoleLetterSpacing,
    maxLogLines,
    bgOpacity,
    bgBlur,
    bgBrightness,
    syncLocalValues,
    prepareForSave,
  };
}
