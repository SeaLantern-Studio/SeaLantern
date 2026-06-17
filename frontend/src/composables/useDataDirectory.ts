import { computed, ref } from "vue";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import { useServerStore } from "@stores/serverStore";
import { useSettingsStore } from "@stores/settingsStore";

export function useDataDirectory() {
  const settingsStore = useSettingsStore();
  const pluginStore = usePluginStore();
  const serverStore = useServerStore();
  const isBusy = ref(false);
  const error = ref<string | null>(null);
  const infoMessage = ref<string | null>(null);

  const status = computed(() => settingsStore.dataDirStatus);

  function clearFeedback() {
    error.value = null;
    infoMessage.value = null;
  }

  async function browseFolder(): Promise<string | null> {
    return systemApi.pickFolder();
  }

  async function refreshStatus() {
    clearFeedback();
    return settingsStore.refreshDataDirStatus();
  }

  async function refreshDependentViews() {
    const results = await Promise.allSettled([
      pluginStore.loadPlugins(),
      serverStore.refreshList(),
    ]);
    for (const result of results) {
      if (result.status === "rejected") {
        console.warn("Failed to refresh state after data directory change:", result.reason);
      }
    }
  }

  async function initialize(path: string) {
    clearFeedback();
    if (!path.trim()) {
      throw new Error(i18n.t("settings.data_dir_required"));
    }

    isBusy.value = true;
    try {
      const result = await settingsStore.initializeDataDir(path.trim());
      await refreshDependentViews();
      infoMessage.value = i18n.t("settings.data_dir_init_success", {
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

  async function change(path: string, migrateExisting = true) {
    clearFeedback();
    if (!path.trim()) {
      throw new Error(i18n.t("settings.data_dir_required"));
    }

    isBusy.value = true;
    try {
      const result = await settingsStore.changeDataDir(path.trim(), migrateExisting);
      await refreshDependentViews();
      infoMessage.value = i18n.t("settings.data_dir_change_success", {
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
    initialize,
    change,
  };
}
