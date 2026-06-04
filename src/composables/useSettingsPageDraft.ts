import { onActivated, onMounted, onUnmounted, ref, watch } from "vue";
import type { AppSettings, SettingsGroup } from "@api/settings";
import { useSettingsStore } from "@stores/settingsStore";

interface UseSettingsPageDraftOptions {
  changedGroups: SettingsGroup[];
  syncLocalValues?: (settings: AppSettings) => void;
  prepareForSave?: (settings: AppSettings) => void;
  emptyImportMessage?: () => string;
}

export function useSettingsPageDraft(options: UseSettingsPageDraftOptions) {
  const settingsStore = useSettingsStore();

  const settings = ref<AppSettings | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const showImportModal = ref(false);
  const showResetConfirm = ref(false);
  const isDirty = ref(false);

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function clearSaveTimer() {
    if (!saveTimeout) {
      return;
    }

    clearTimeout(saveTimeout);
    saveTimeout = null;
  }

  function clearError() {
    error.value = null;
  }

  function setError(message: string | null) {
    error.value = message;
  }

  function applyStoreSnapshot(source: AppSettings = settingsStore.settings) {
    const snapshot = settingsStore.cloneSettings(source);
    settings.value = snapshot;
    options.syncLocalValues?.(snapshot);
    isDirty.value = false;
  }

  async function loadSettings() {
    loading.value = true;
    clearError();

    try {
      await settingsStore.ensureLoaded();
      applyStoreSnapshot();
    } catch (cause) {
      setError(String(cause));
    } finally {
      loading.value = false;
    }
  }

  async function saveSettings() {
    if (!settings.value) {
      return;
    }

    clearError();
    options.prepareForSave?.(settings.value);

    try {
      await settingsStore.saveSettingsWithDiff(settings.value);
      applyStoreSnapshot();
    } catch (cause) {
      setError(String(cause));
    }
  }

  function markChanged() {
    isDirty.value = true;
    clearSaveTimer();
    saveTimeout = setTimeout(() => {
      void saveSettings();
      saveTimeout = null;
    }, 500);
  }

  async function resetSettings() {
    try {
      const nextSettings = await settingsStore.resetSettings(options.changedGroups);
      applyStoreSnapshot(nextSettings);
      showResetConfirm.value = false;
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function importSettings(json: string): Promise<boolean> {
    if (!json.trim()) {
      setError(options.emptyImportMessage?.() ?? "");
      return false;
    }

    try {
      const nextSettings = await settingsStore.importSettingsJson(json, options.changedGroups);
      applyStoreSnapshot(nextSettings);
      showImportModal.value = false;
      return true;
    } catch (cause) {
      setError(String(cause));
      return false;
    }
  }

  function replaceSettings(source: AppSettings, changedGroups: SettingsGroup[] = []) {
    const nextSettings = settingsStore.replaceSettings(source, changedGroups);
    applyStoreSnapshot(nextSettings);
    return nextSettings;
  }

  watch(
    () => settingsStore.settings,
    (nextSettings, previousSettings) => {
      if (nextSettings === previousSettings || isDirty.value) {
        return;
      }

      applyStoreSnapshot(nextSettings);
    },
  );

  onMounted(() => {
    void loadSettings();
  });

  onActivated(() => {
    if (!settingsStore.isLoaded) {
      void loadSettings();
      return;
    }

    if (!isDirty.value) {
      applyStoreSnapshot();
    }
  });

  onUnmounted(() => {
    clearSaveTimer();
  });

  return {
    settings,
    loading,
    error,
    showImportModal,
    showResetConfirm,
    isDirty,
    clearError,
    setError,
    loadSettings,
    saveSettings,
    markChanged,
    resetSettings,
    importSettings,
    applyStoreSnapshot,
    replaceSettings,
  };
}
