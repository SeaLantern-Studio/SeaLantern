import { computed, onMounted, onUnmounted, watch, type Ref } from "vue";
import { onBeforeRouteLeave, onBeforeRouteUpdate } from "vue-router";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { useAutoSaveSettings } from "./useAutoSaveSettings";
import {
  serializeSettingsRecord,
  type PluginSettingsRecord,
} from "./pluginSettingsShared";
import type { PluginInfo } from "@type/plugin";

interface UsePluginCategoryLifecycleOptions {
  pluginId: () => string;
  plugin: { value: PluginInfo | null };
  pluginSettingsForm: PluginSettingsRecord;
  mainSettingsSnapshot: Ref<string>;
  isInitializingForms: Ref<boolean>;
  loadPluginData: () => Promise<void>;
  syncDependentAutoSaves: () => void;
  dependentSaving: Ref<boolean>;
  flushDependentAutoSaves: () => Promise<void>;
  stopDependentAutoSaves: () => void;
  setPluginSettings: (pluginId: string, payload: Record<string, unknown>) => Promise<void>;
}

export function usePluginCategoryLifecycle(options: UsePluginCategoryLifecycleOptions) {
  const mainAutoSave = useAutoSaveSettings({
    source: options.pluginSettingsForm,
    snapshot: options.mainSettingsSnapshot,
    enabled: () => Boolean(options.plugin.value) && !options.isInitializingForms.value,
    save: async (payload) => {
      const pluginId = options.plugin.value?.manifest.id;
      if (!pluginId) {
        return;
      }

      await options.setPluginSettings(pluginId, payload);
    },
    onError: (error) => {
      pluginLogger.error("PluginCategorySettings", "Failed to auto-save plugin settings", {
        pluginId: options.pluginId(),
        error,
      });
    },
  });

  const saving = computed(() => mainAutoSave.saving.value || options.dependentSaving.value);

  async function flushPendingAutoSaves() {
    await mainAutoSave.flush();
    await options.flushDependentAutoSaves();
  }

  function finishFormInitialization() {
    options.syncDependentAutoSaves();
    options.mainSettingsSnapshot.value = serializeSettingsRecord({ ...options.pluginSettingsForm });
    options.isInitializingForms.value = false;
  }

  onMounted(() => {
    void options.loadPluginData();
  });

  watch(
    () => options.pluginId(),
    () => {
      void options.loadPluginData();
    },
  );

  onBeforeRouteLeave(async () => {
    await flushPendingAutoSaves();
  });

  onBeforeRouteUpdate(async () => {
    await flushPendingAutoSaves();
  });

  onUnmounted(() => {
    options.stopDependentAutoSaves();
  });

  return {
    saving,
    finishFormInitialization,
    flushPendingAutoSaves,
  };
}
