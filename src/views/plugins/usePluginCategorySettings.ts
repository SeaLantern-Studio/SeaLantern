import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";
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
  serializeSettingsRecord,
  type PluginSettingsRecord,
  updatePluginSettingsField,
} from "@views/plugins/pluginSettingsShared";
import { useAutoSaveSettings } from "@views/plugins/useAutoSaveSettings";

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

  async function loadDependentPlugins() {
    if (!plugin.value || !showDependents.value) {
      dependentPlugins.value = [];
      clearSettingsRecord(dependentSettingsForms);
      clearSettingsRecord(dependentSettingsSnapshots);
      return;
    }

    const candidates = findDependentPlugins(pluginStore.plugins, options.pluginId());
    const results = await Promise.all(
      candidates
        .filter((candidate) => candidate.manifest.settings?.length)
        .map(async (candidate) => ({
          plugin: candidate,
          form: buildPluginSettingsForm(
            candidate.manifest.settings,
            await pluginStore.getPluginSettings(candidate.manifest.id),
          ),
        })),
    );

    dependentPlugins.value = results.map((item) => item.plugin);
    clearSettingsRecord(dependentSettingsForms);
    clearSettingsRecord(dependentSettingsSnapshots);

    for (const { plugin: depPlugin, form } of results) {
      dependentSettingsForms[depPlugin.manifest.id] = form;
      dependentSettingsSnapshots[depPlugin.manifest.id] = serializeSettingsRecord(form);
    }
  }

  const mainAutoSave = useAutoSaveSettings({
    source: settingsForm,
    snapshot: mainSettingsSnapshot,
    enabled: () => Boolean(plugin.value) && !isInitializingForms.value,
    save: async (payload) => {
      const pluginId = plugin.value?.manifest.id;
      if (!pluginId) {
        return;
      }

      await pluginStore.setPluginSettings(pluginId, payload);
    },
    onError: (error) => {
      pluginLogger.error("PluginCategorySettings", "Failed to auto-save plugin settings", {
        pluginId: options.pluginId(),
        error,
      });
    },
  });

  const dependentAutoSaves = new Map<string, ReturnType<typeof useAutoSaveSettings>>();

  const saving = computed(() => {
    if (mainAutoSave.saving.value) {
      return true;
    }

    return Array.from(dependentAutoSaves.values()).some((entry) => entry.saving.value);
  });

  function syncDependentAutoSaves() {
    const activeIds = new Set(dependentPlugins.value.map((depPlugin) => depPlugin.manifest.id));

    for (const [pluginId, autoSave] of dependentAutoSaves.entries()) {
      if (!activeIds.has(pluginId)) {
        autoSave.stop();
        dependentAutoSaves.delete(pluginId);
      }
    }

    for (const depPlugin of dependentPlugins.value) {
      const pluginId = depPlugin.manifest.id;
      if (dependentAutoSaves.has(pluginId)) {
        continue;
      }

      if (!dependentSettingsForms[pluginId]) {
        dependentSettingsForms[pluginId] = {};
      }

      if (dependentSettingsSnapshots[pluginId] === undefined) {
        dependentSettingsSnapshots[pluginId] = serializeSettingsRecord(
          dependentSettingsForms[pluginId],
        );
      }

      dependentAutoSaves.set(
        pluginId,
        useAutoSaveSettings({
          source: dependentSettingsForms[pluginId],
          snapshot: computed({
            get: () => dependentSettingsSnapshots[pluginId] ?? "{}",
            set: (value) => {
              dependentSettingsSnapshots[pluginId] = value;
            },
          }),
          enabled: () => !isInitializingForms.value,
          save: async (payload) => {
            await pluginStore.setPluginSettings(pluginId, payload);
          },
          onError: (error) => {
            pluginLogger.error(
              "PluginCategorySettings",
              "Failed to auto-save dependent plugin settings",
              {
                pluginId,
                ownerPluginId: options.pluginId(),
                error,
              },
            );
          },
        }),
      );
    }
  }

  async function flushPendingAutoSaves() {
    await mainAutoSave.flush();
    await Promise.all(Array.from(dependentAutoSaves.values()).map((entry) => entry.flush()));
  }

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
      syncDependentAutoSaves();
      mainSettingsSnapshot.value = serializeSettingsRecord({ ...settingsForm });
      isInitializingForms.value = false;
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

  onMounted(() => {
    void loadPluginData();
  });

  watch(
    () => pluginStore.plugins,
    (newPlugins) => {
      if (newPlugins.length > 0 && !plugin.value) {
        void loadPluginData();
      }
    },
    { deep: false },
  );

  watch(
    () => options.pluginId(),
    () => {
      void loadPluginData();
    },
  );

  onBeforeRouteLeave(async () => {
    await flushPendingAutoSaves();
  });

  onBeforeRouteUpdate(async () => {
    await flushPendingAutoSaves();
  });

  onUnmounted(() => {
    for (const autoSave of dependentAutoSaves.values()) {
      autoSave.stop();
    }

    dependentAutoSaves.clear();
  });

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
