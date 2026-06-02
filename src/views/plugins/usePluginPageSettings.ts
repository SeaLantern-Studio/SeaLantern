import { computed, onMounted, reactive, ref, watch } from "vue";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import type { PluginInfo } from "@type/plugin";
import {
  applyPluginPreset,
  buildPluginSettingsForm,
  clearSettingsRecord,
  findDependentPlugins,
  getPluginFieldOptions,
  getPluginFieldSelectValue,
  getPluginFieldStringValue,
  resetPluginSettingsForm,
  type PluginSettingsRecord,
  updatePluginSettingsField,
} from "@views/plugins/pluginSettingsShared";

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

  async function loadDependentPlugins() {
    dependentPlugins.value = [];
    clearSettingsRecord(dependentSettingsForms);

    const candidates = findDependentPlugins(pluginStore.plugins, options.pluginId());

    const settingsPromises = candidates
      .filter((candidate) => candidate.manifest.settings?.length)
      .map(async (candidate) => {
        return {
          plugin: candidate,
          form: buildPluginSettingsForm(
            candidate.manifest.settings,
            await pluginStore.getPluginSettings(candidate.manifest.id),
          ),
        };
      });

    const results = await Promise.all(settingsPromises);
    for (const { plugin: dependentPlugin, form } of results) {
      dependentPlugins.value.push(dependentPlugin);
      dependentSettingsForms[dependentPlugin.manifest.id] = form;
    }
  }

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

    try {
      const settingsToSave = applyPluginPreset(settingsForm, presetKey, presets[presetKey]);
      await pluginStore.setPluginSettings(pluginId, settingsToSave);
    } catch (error) {
      pluginLogger.error("PluginPageSettings", "Failed to apply plugin preset", {
        pluginId,
        presetKey,
        error,
      });
      throw error;
    }
  }

  async function saveSettings() {
    if (!plugin.value) {
      return;
    }

    saving.value = true;
    try {
      await pluginStore.setPluginSettings(options.pluginId(), { ...settingsForm });
      if (isThemeProvider.value) {
        await pluginStore.applyThemeProviderSettings(options.pluginId());
      }

      const dependentSaves = dependentPlugins.value.map(async (dependentPlugin) => {
        const dependentForm = dependentSettingsForms[dependentPlugin.manifest.id];
        if (!dependentForm) {
          return;
        }

        await pluginStore.setPluginSettings(dependentPlugin.manifest.id, { ...dependentForm });
      });
      await Promise.all(dependentSaves);
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
