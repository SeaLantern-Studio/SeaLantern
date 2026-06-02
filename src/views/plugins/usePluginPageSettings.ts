import { computed, onMounted, reactive, ref, watch } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import type { PluginInfo } from "@type/plugin";

type PluginSettingValue = string | number | boolean | null;
type SettingsForm = Record<string, PluginSettingValue>;
type DependentSettingsForms = Record<string, SettingsForm>;

interface UsePluginPageSettingsOptions {
  pluginId: () => string;
}

export function usePluginPageSettings(options: UsePluginPageSettingsOptions) {
  const pluginStore = usePluginStore();
  const plugin = ref<PluginInfo | null>(null);
  const settingsForm = reactive<SettingsForm>({});
  const saving = ref(false);
  const loading = ref(true);
  const dependentPlugins = ref<PluginInfo[]>([]);
  const dependentSettingsForms = reactive<DependentSettingsForms>({});

  const pluginPresets = computed(() => plugin.value?.manifest.presets ?? null);
  const isThemeProvider = computed(() => {
    return plugin.value?.manifest.capabilities?.includes("theme-provider") ?? false;
  });

  function getDefaultValue(type: string): PluginSettingValue {
    switch (type) {
      case "boolean":
        return false;
      case "number":
        return 0;
      case "select":
        return "";
      default:
        return "";
    }
  }

  function getFieldStringValue(value: PluginSettingValue | undefined): string {
    return value == null ? "" : String(value);
  }

  function getFieldSelectValue(value: PluginSettingValue | undefined): string | number | undefined {
    if (typeof value === "string" || typeof value === "number") {
      return value;
    }
    return undefined;
  }

  function getFieldOptions(
    options: Array<{ value: string; label: string }> | undefined,
  ): Array<{ value: string; label: string }> {
    return options ?? [];
  }

  function updateSettingsField(form: SettingsForm, key: string, value: string | number | boolean) {
    form[key] = value;
  }

  async function loadDependentPlugins() {
    dependentPlugins.value = [];
    Object.keys(dependentSettingsForms).forEach((key) => delete dependentSettingsForms[key]);

    const candidates = pluginStore.plugins.filter((candidate) => {
      if (candidate.state !== "enabled") {
        return false;
      }
      if (candidate.manifest.id === options.pluginId()) {
        return false;
      }
      const dependencies = candidate.manifest.dependencies || [];
      return dependencies.some((dependency: string | { id: string }) => {
        const dependencyId = typeof dependency === "string" ? dependency : dependency.id;
        return dependencyId === options.pluginId();
      });
    });

    const settingsPromises = candidates
      .filter((candidate) => candidate.manifest.settings?.length)
      .map(async (candidate) => {
        const candidateSettings = await pluginStore.getPluginSettings(candidate.manifest.id);
        const form: SettingsForm = {};
        for (const field of candidate.manifest.settings || []) {
          form[field.key] =
            candidateSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
        }
        return { plugin: candidate, form };
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
      if (!plugin.value) {
        return;
      }

      const savedSettings = await pluginStore.getPluginSettings(options.pluginId());
      Object.keys(settingsForm).forEach((key) => delete settingsForm[key]);
      for (const field of plugin.value.manifest.settings || []) {
        settingsForm[field.key] =
          savedSettings[field.key] ?? field.default ?? getDefaultValue(field.type);
      }

      await loadDependentPlugins();
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

    const presetData = presets[presetKey];
    const settingsToSave: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(presetData)) {
      if (key === "name") {
        continue;
      }
      settingsForm[key] = value as string | number | boolean;
      settingsToSave[key] = value;
    }

    await pluginStore.setPluginSettings(pluginId, settingsToSave);
    await pluginStore.applyThemeProviderSettings(pluginId);
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
        if (pluginStore.hasCapability(dependentPlugin.manifest.id, "theme-widgets-provider")) {
          await pluginStore.applyThemeWidgetsProviderSettings(dependentPlugin.manifest.id);
        }
      });
      await Promise.all(dependentSaves);
    } finally {
      saving.value = false;
    }
  }

  function resetToDefault() {
    if (!plugin.value?.manifest.settings) {
      return;
    }
    for (const field of plugin.value.manifest.settings) {
      settingsForm[field.key] = field.default ?? getDefaultValue(field.type);
    }
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
    getFieldStringValue,
    getFieldSelectValue,
    getFieldOptions,
    updateSettingsField,
    applyPreset,
    saveSettings,
    resetToDefault,
  };
}
