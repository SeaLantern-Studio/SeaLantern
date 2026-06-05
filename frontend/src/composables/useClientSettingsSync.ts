import { onMounted, onUnmounted, watch } from "vue";
import { useSettingsStore } from "@stores/settingsStore";

export function useClientSettingsSync() {
  const settingsStore = useSettingsStore();
  let systemThemeQuery: MediaQueryList | null = null;

  function applyCurrentSettings() {
    return settingsStore.queueClientSettingsApply(settingsStore.settings);
  }

  function handleSystemThemeChange() {
    if (settingsStore.settings.theme !== "auto") {
      return;
    }

    void applyCurrentSettings();
  }

  onMounted(async () => {
    await settingsStore.ensureLoaded();
    await applyCurrentSettings();

    systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
    systemThemeQuery.addEventListener("change", handleSystemThemeChange);
  });

  onUnmounted(() => {
    if (!systemThemeQuery) {
      return;
    }

    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
    systemThemeQuery = null;
  });

  watch(
    () => settingsStore.settings,
    (nextSettings, previousSettings) => {
      if (nextSettings === previousSettings) {
        return;
      }

      void settingsStore.queueClientSettingsApply(nextSettings);
    },
    { deep: true },
  );
}
