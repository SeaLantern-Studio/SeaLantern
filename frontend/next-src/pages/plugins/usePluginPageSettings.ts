import { computed, onMounted, reactive, ref, watch } from "vue";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import type { PluginInfo } from "@type/plugin";
import {
  applyPluginPreset,
  buildPluginSettingsForm,
  clearSettingsRecord,
  getPluginFieldOptions,
  getPluginFieldSelectValue,
  getPluginFieldStringValue,
  resetPluginSettingsForm,
  type PluginSettingsRecord,
  updatePluginSettingsField,
} from "./pluginSettingsShared";
import { usePluginDependentSettings } from "./usePluginDependentSettings";
import { usePluginSettingsPersistence } from "./usePluginSettingsPersistence";

type DependentSettingsForms = Record<string, PluginSettingsRecord>;

interface UsePluginPageSettingsOptions {
  pluginId: () => string;
}

export function usePluginPageSettings(options: UsePluginPageSettingsOptions) {
  const pluginStore = usePluginStore();
  const plugin = ref<PluginInfo | null>(null);
  const settingsForm = reactive<PluginSettingsRecord>({});
  const saving = ref(false);
  const loading = ref(true);
  const dependentPlugins = ref<PluginInfo[]>([]);
  const dependentSettingsForms = reactive<DependentSettingsForms>({});

  const pluginPresets = computed(() => plugin.value?.manifest.presets ?? null);
  const isThemeProvider = computed(() => {
    return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
  });

  const { loadDependentPlugins } = usePluginDependentSettings({
    pluginId: options.pluginId,
    plugins: () => pluginStore.plugins,
    showDependents: () => true,
    hasPlugin: () => Boolean(plugin.value),
    getPluginSettings: pluginStore.getPluginSettings,
    dependentPlugins,
    dependentSettingsForms,
    dependentSettingsSnapshots: {},
  });

  const settingsPersistence = usePluginSettingsPersistence({
    ownerPluginId: options.pluginId,
    plugin: () => plugin.value,
    dependentPlugins: () => dependentPlugins.value,
    dependentSettingsForms,
    setPluginSettings: pluginStore.setPluginSettings,
  });

  async function loadPlugin() {
    loading.value = true;
    try {
      if (!pluginStore.plugins.length) {
        await pluginStore.loadPlugins();
      }

      plugin.value =
        pluginStore.plugins.find((item) => item.manifest.id === options.pluginId()) || null;
      clearSettingsRecord(settingsForm);
      clearSettingsRecord(dependentSettingsForms);
      dependentPlugins.value = [];

      if (!plugin.value) {
        return;
      }

      Object.assign(
        settingsForm,
        buildPluginSettingsForm(
          plugin.value.manifest.settings,
          await pluginStore.getPluginSettings(options.pluginId()),
        ),
      );

      await loadDependentPlugins();
    } catch (error) {
      plugin.value = null;
      dependentPlugins.value = [];
      clearSettingsRecord(settingsForm);
      clearSettingsRecord(dependentSettingsForms);
      pluginLogger.error("PluginPageSettings", "Failed to load plugin settings page", {
        pluginId: options.pluginId(),
        error,
      });
    } finally {
      loading.value = false;
    }
  }

  async function applyPreset(presetKey: string) {
    const presets = pluginPresets.value;
    const pluginId = plugin.value?.manifest.id;
    if (!presets || !presets[presetKey] || !pluginId) {
      return;
    }

    const settingsToSave = applyPluginPreset(settingsForm, presetKey, presets[presetKey]);
    await settingsPersistence.applyPreset(pluginId, presetKey, settingsToSave);
  }

  async function saveSettings() {
    if (!plugin.value) {
      return;
    }

    saving.value = true;
    try {
      await settingsPersistence.savePluginSettings(settingsForm);
    } catch (error) {
      pluginLogger.error("PluginPageSettings", "Failed to save plugin settings", {
        pluginId: options.pluginId(),
        error,
      });
      throw error;
    } finally {
      saving.value = false;
    }
  }

  function resetToDefault() {
    if (!plugin.value?.manifest.settings) {
      return;
    }

    resetPluginSettingsForm(settingsForm, plugin.value.manifest.settings);
  }

  onMounted(() => {
    void loadPlugin();
  });

  watch(
    () => options.pluginId(),
    () => {
      void loadPlugin();
    },
  );

  return {
    plugin,
    settingsForm,
    saving,
    loading,
    dependentPlugins,
    dependentSettingsForms,
    pluginPresets,
    isThemeProvider,
    getFieldStringValue: getPluginFieldStringValue,
    getFieldSelectValue: getPluginFieldSelectValue,
    getFieldOptions: getPluginFieldOptions,
    updateSettingsField: updatePluginSettingsField,
    applyPreset,
    saveSettings,
    resetToDefault,
  };
}
