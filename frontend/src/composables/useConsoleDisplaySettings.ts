import { ref, watch } from "vue";
import { useSettingsStore } from "@stores/settingsStore";

export function useConsoleDisplaySettings() {
  const settingsStore = useSettingsStore();

  const consoleFontSize = ref(13);
  const consoleFontFamily = ref("");
  const consoleLetterSpacing = ref(0);
  const maxLogLines = ref(5000);

  function applyConsoleSettings(settings: {
    console_font_size: number;
    console_font_family: string;
    console_letter_spacing: number;
    max_log_lines: number;
  }) {
    consoleFontSize.value = settings.console_font_size;
    consoleFontFamily.value = settings.console_font_family || "";
    consoleLetterSpacing.value = settings.console_letter_spacing || 0;
    maxLogLines.value = Math.max(100, settings.max_log_lines || 5000);
  }

  watch(
    () => settingsStore.settings,
    (settings) => {
      applyConsoleSettings(settings);
    },
    { deep: true, immediate: true },
  );

  return {
    consoleFontSize,
    consoleFontFamily,
    consoleLetterSpacing,
    maxLogLines,
    applyConsoleSettings,
  };
}
