import { computed } from "vue";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  serializeSettingsRecord,
  type PluginSettingsRecord,
} from "./pluginSettingsShared";
import { useAutoSaveSettings } from "./useAutoSaveSettings";
import type { PluginInfo } from "@type/plugin";

interface UseDependentAutoSavesOptions {
  ownerPluginId: () => string;
  dependentPlugins: { value: PluginInfo[] };
  dependentSettingsForms: Record<string, PluginSettingsRecord>;
  dependentSettingsSnapshots: Record<string, string>;
  isInitializingForms: () => boolean;
  setPluginSettings: (pluginId: string, payload: Record<string, unknown>) => Promise<void>;
}

export function useDependentAutoSaves(options: UseDependentAutoSavesOptions) {
  const dependentAutoSaves = new Map<string, ReturnType<typeof useAutoSaveSettings>>();

  const saving = computed(() => {
    return Array.from(dependentAutoSaves.values()).some((entry) => entry.saving.value);
  });

  function syncDependentAutoSaves() {
    const activeIds = new Set(options.dependentPlugins.value.map((plugin) => plugin.manifest.id));

    for (const [pluginId, autoSave] of dependentAutoSaves.entries()) {
      if (!activeIds.has(pluginId)) {
        autoSave.stop();
        dependentAutoSaves.delete(pluginId);
      }
    }

    for (const dependentPlugin of options.dependentPlugins.value) {
      const pluginId = dependentPlugin.manifest.id;
      if (dependentAutoSaves.has(pluginId)) {
        continue;
      }

      if (!options.dependentSettingsForms[pluginId]) {
        options.dependentSettingsForms[pluginId] = {};
      }

      if (options.dependentSettingsSnapshots[pluginId] === undefined) {
        options.dependentSettingsSnapshots[pluginId] = serializeSettingsRecord(
          options.dependentSettingsForms[pluginId],
        );
      }

      dependentAutoSaves.set(
        pluginId,
        useAutoSaveSettings({
          source: options.dependentSettingsForms[pluginId],
          snapshot: computed({
            get: () => options.dependentSettingsSnapshots[pluginId] ?? "{}",
            set: (value) => {
              options.dependentSettingsSnapshots[pluginId] = value;
            },
          }),
          enabled: () => !options.isInitializingForms(),
          save: async (payload) => {
            await options.setPluginSettings(pluginId, payload);
          },
          onError: (error) => {
            pluginLogger.error(
              "PluginCategorySettings",
              "Failed to auto-save dependent plugin settings",
              {
                pluginId,
                ownerPluginId: options.ownerPluginId(),
                error,
              },
            );
          },
        }),
      );
    }
  }

  async function flushDependentAutoSaves() {
    await Promise.all(Array.from(dependentAutoSaves.values()).map((entry) => entry.flush()));
  }

  function stopDependentAutoSaves() {
    for (const autoSave of dependentAutoSaves.values()) {
      autoSave.stop();
    }

    dependentAutoSaves.clear();
  }

  return {
    dependentSaving: saving,
    syncDependentAutoSaves,
    flushDependentAutoSaves,
    stopDependentAutoSaves,
  };
}
