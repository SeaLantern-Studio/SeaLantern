import { computed, ref } from "vue";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import { useSettingsStore } from "@stores/settingsStore";

async function browseFolder(): Promise<string | null> {
  return systemApi.pickFolder();
}

export function usePluginDirectory() {
  const settingsStore = useSettingsStore();
  const pluginStore = usePluginStore();
  const isBusy = ref(false);
  const error = ref<string | null>(null);
  const infoMessage = ref<string | null>(null);

  const status = computed(() => settingsStore.pluginDirStatus);

  function clearFeedback() {
    error.value = null;
    infoMessage.value = null;
  }

  async function refreshStatus() {
    clearFeedback();
    return settingsStore.refreshPluginDirStatus();
  }

  async function refreshDependentViews() {
    try {
      await pluginStore.loadPlugins();
    } catch (cause) {
      console.warn("Failed to refresh plugins after plugin directory change:", cause);
    }
  }

  async function change(path: string, migrateExisting = true) {
    clearFeedback();
    if (!path.trim()) {
      throw new Error(i18n.t("settings.plugin_dir_required"));
    }

    isBusy.value = true;
    try {
      const result = await settingsStore.changePluginDir(path.trim(), migrateExisting);
      await refreshDependentViews();
      infoMessage.value = i18n.t("settings.plugin_dir_change_success", {
        count: result.migrated_entries.length,
      });
      return result;
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : String(cause);
      throw cause;
    } finally {
      isBusy.value = false;
    }
  }

  return {
    status,
    isBusy,
    error,
    infoMessage,
    clearFeedback,
    browseFolder,
    refreshStatus,
    change,
  };
}
