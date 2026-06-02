import { reactive, ref } from "vue";
import { normalizeAppError, resolveErrorMessage } from "@utils/appError";
import type { PluginInfo } from "@type/plugin";

type PluginSettingValue = string | number | boolean;

interface PluginSettingsActions {
  getPluginSettings: (pluginId: string) => Promise<Record<string, unknown>>;
  setPluginSettings: (pluginId: string, settings: Record<string, unknown>) => Promise<void>;
  logError: (message: string, details?: unknown) => void;
}

export function usePluginSettingsDialog(actions: PluginSettingsActions) {
  const showSettingsModal = ref(false);
  const currentSettingsPlugin = ref<PluginInfo | null>(null);
  const settingsForm = reactive<Record<string, PluginSettingValue>>({});
  const savingSettings = ref(false);

  function getDefaultValue(type: string): string | number | boolean {
    switch (type) {
      case "boolean":
        return false;
      case "number":
        return 0;
      default:
        return "";
    }
  }

  async function openSettings(plugin: PluginInfo) {
    currentSettingsPlugin.value = plugin;
    const savedSettings = await actions.getPluginSettings(plugin.manifest.id);
    Object.keys(settingsForm).forEach((key) => delete settingsForm[key]);
    if (plugin.manifest.settings) {
      for (const field of plugin.manifest.settings) {
        settingsForm[field.key] =
          savedSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
      }
    }
    showSettingsModal.value = true;
  }

  function closeSettings() {
    showSettingsModal.value = false;
    currentSettingsPlugin.value = null;
  }

  async function saveSettings(): Promise<string | null> {
    if (!currentSettingsPlugin.value) {
      return null;
    }

    savingSettings.value = true;
    try {
      await actions.setPluginSettings(currentSettingsPlugin.value.manifest.id, {
        ...settingsForm,
      });

      closeSettings();
      return null;
    } catch (error) {
      const normalized = normalizeAppError(error);
      actions.logError("插件设置保存失败", normalized);
      return normalized.message || resolveErrorMessage(normalized.code, normalized.args);
    } finally {
      savingSettings.value = false;
    }
  }

  return {
    showSettingsModal,
    currentSettingsPlugin,
    settingsForm,
    savingSettings,
    openSettings,
    closeSettings,
    saveSettings,
  };
}
