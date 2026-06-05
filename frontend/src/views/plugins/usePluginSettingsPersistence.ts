import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginInfo } from "@type/plugin";
import type { PluginSettingsRecord } from "@views/plugins/pluginSettingsShared";

interface UsePluginSettingsPersistenceOptions {
  ownerPluginId: () => string;
  plugin: () => PluginInfo | null;
  dependentPlugins: () => PluginInfo[];
  dependentSettingsForms: Record<string, PluginSettingsRecord>;
  setPluginSettings: (pluginId: string, settings: Record<string, unknown>) => Promise<void>;
}

export function usePluginSettingsPersistence(options: UsePluginSettingsPersistenceOptions) {
  async function savePluginSettings(settingsForm: PluginSettingsRecord) {
    const plugin = options.plugin();
    if (!plugin) {
      return;
    }

    await options.setPluginSettings(options.ownerPluginId(), { ...settingsForm });

    const dependentSaves = options.dependentPlugins().map(async (dependentPlugin) => {
      const dependentForm = options.dependentSettingsForms[dependentPlugin.manifest.id];
      if (!dependentForm) {
        return;
      }

      await options.setPluginSettings(dependentPlugin.manifest.id, { ...dependentForm });
    });

    await Promise.all(dependentSaves);
  }

  async function applyPreset(
    pluginId: string,
    presetKey: string,
    payload: Record<string, unknown>,
  ) {
    try {
      await options.setPluginSettings(pluginId, payload);
    } catch (error) {
      pluginLogger.error("PluginPageSettings", "Failed to apply plugin preset", {
        pluginId,
        presetKey,
        error,
      });
      throw error;
    }
  }

  return {
    savePluginSettings,
    applyPreset,
  };
}
