import { computed, reactive, ref, watch } from "vue";
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
  serializeSettingsRecord,
  type PluginSettingsRecord,
  updatePluginSettingsField,
} from "./pluginSettingsShared";
import { useDependentAutoSaves } from "./useDependentAutoSaves";
import { usePluginCategoryLifecycle } from "./usePluginCategoryLifecycle";
import { usePluginDependentSettings } from "./usePluginDependentSettings";

interface UsePluginCategorySettingsOptions {
  pluginId: () => string;
}

export function usePluginCategorySettings(options: UsePluginCategorySettingsOptions) {
  const pluginStore = usePluginStore();

  const plugin = ref<PluginInfo | null>(null);
  const settingsForm = reactive<PluginSettingsRecord>({});
  const dependentSettingsForms = reactive<Record<string, PluginSettingsRecord>>({});
  const dependentPlugins = ref<PluginInfo[]>([]);
  const loading = ref(true);
  const isInitializingForms = ref(false);
  const mainSettingsSnapshot = ref("{}");
  const dependentSettingsSnapshots = reactive<Record<string, string>>({});

  const sidebarConfig = computed(() => plugin.value?.manifest.sidebar);
  const categoryLabel = computed(
    () => sidebarConfig.value?.label || plugin.value?.manifest.name || "",
  );
  const showDependents = computed(() => sidebarConfig.value?.show_dependents !== false);
  const pluginPresets = computed(() => plugin.value?.manifest.presets ?? null);
  const isThemeProvider = computed(() => {
    return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
  });

  const { loadDependentPlugins } = usePluginDependentSettings({
    pluginId: options.pluginId,
    plugins: () => pluginStore.plugins,
    showDependents: () => showDependents.value,
    hasPlugin: () => Boolean(plugin.value),
    getPluginSettings: pluginStore.getPluginSettings,
    dependentPlugins,
    dependentSettingsForms,
    dependentSettingsSnapshots,
  });

  const {
    dependentSaving,
    syncDependentAutoSaves,
    flushDependentAutoSaves,
    stopDependentAutoSaves,
  } = useDependentAutoSaves({
    ownerPluginId: options.pluginId,
    dependentPlugins,
    dependentSettingsForms,
    dependentSettingsSnapshots,
    isInitializingForms: () => isInitializingForms.value,
    setPluginSettings: pluginStore.setPluginSettings,
  });

  const { saving, finishFormInitialization, flushPendingAutoSaves } = usePluginCategoryLifecycle({
    pluginId: options.pluginId,
    plugin,
    pluginSettingsForm: settingsForm,
    mainSettingsSnapshot,
    isInitializingForms,
    loadPluginData,
    syncDependentAutoSaves,
    dependentSaving,
    flushDependentAutoSaves,
    stopDependentAutoSaves,
    setPluginSettings: pluginStore.setPluginSettings,
  });

  async function loadPluginData() {
    loading.value = true;
    await flushPendingAutoSaves();
    isInitializingForms.value = true;

    clearSettingsRecord(settingsForm);
    clearSettingsRecord(dependentSettingsForms);
    clearSettingsRecord(dependentSettingsSnapshots);
    dependentPlugins.value = [];

    try {
      if (!pluginStore.plugins.length) {
        await pluginStore.loadPlugins();
      }

      const found =
        pluginStore.plugins.find((item) => item.manifest.id === options.pluginId()) ?? null;
      plugin.value = found;
      if (!found) {
        return;
      }

      Object.assign(
        settingsForm,
        buildPluginSettingsForm(
          found.manifest.settings,
          await pluginStore.getPluginSettings(options.pluginId()),
        ),
      );

      await loadDependentPlugins();
    } catch (error) {
      plugin.value = null;
      dependentPlugins.value = [];
      clearSettingsRecord(settingsForm);
      clearSettingsRecord(dependentSettingsForms);
      clearSettingsRecord(dependentSettingsSnapshots);
      pluginLogger.error("PluginCategorySettings", "Failed to load category settings", {
        pluginId: options.pluginId(),
        error,
      });
    } finally {
      finishFormInitialization();
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
      const payload = applyPluginPreset(settingsForm, presetKey, presets[presetKey]);
      await pluginStore.setPluginSettings(pluginId, payload);
      mainSettingsSnapshot.value = serializeSettingsRecord({ ...settingsForm });
    } catch (error) {
      pluginLogger.error("PluginCategorySettings", "Failed to apply preset", {
        pluginId,
        presetKey,
        error,
      });
    }
  }

  function resetToDefault() {
    if (!plugin.value?.manifest.settings) {
      return;
    }

    resetPluginSettingsForm(settingsForm, plugin.value.manifest.settings);
  }

  watch(
    () => pluginStore.plugins,
    (newPlugins) => {
      if (newPlugins.length > 0 && !plugin.value) {
        void loadPluginData();
      }
    },
    { deep: false },
  );

  return {
    plugin,
    settingsForm,
    dependentSettingsForms,
    dependentPlugins,
    loading,
    saving,
    categoryLabel,
    showDependents,
    pluginPresets,
    isThemeProvider,
    getFieldStringValue: getPluginFieldStringValue,
    getFieldSelectValue: getPluginFieldSelectValue,
    getFieldOptions: getPluginFieldOptions,
    updateSettingsField: updatePluginSettingsField,
    applyPreset,
    resetToDefault,
  };
}
