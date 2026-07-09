import { computed, ref } from "vue";
import { useUpdateStore } from "@stores/updateStore";
import { useI18nStore } from "@stores/i18nStore";
import { useSettingsStore } from "@stores/settingsStore";
import { usePluginStore } from "@stores/pluginStore";
import { useServerStore } from "@stores/serverStore";
import { useDataDirectory } from "@composables/useDataDirectory";

export function useAppBootstrap() {
  const showSplash = ref(true);
  const isInitializing = ref(true);
  const showTermsDialog = ref(false);
  const showDataDirDialog = ref(false);
  const updateStore = useUpdateStore();
  const i18nStore = useI18nStore();
  const settingsStore = useSettingsStore();
  const pluginStore = usePluginStore();
  const serverStore = useServerStore();
  const dataDirectory = useDataDirectory();

  async function initializeApp() {
    try {
      await settingsStore.loadSettings();
      await i18nStore.loadLanguageSetting();

      try {
        await pluginStore.loadPlugins();
      } catch (pluginError) {
        console.warn("Failed to load plugins during startup:", pluginError);
      }

      try {
        await serverStore.refreshList();
      } catch (serverError) {
        console.warn("Failed to load servers during startup:", serverError);
      }
    } catch (error) {
      console.error("Failed to load settings during startup:", error);
    } finally {
      isInitializing.value = false;
    }
  }

  async function handleAgreeTerms() {
    try {
      await settingsStore.updatePartial({ agreed_to_terms: true });
      showTermsDialog.value = false;
      if (settingsStore.dataDirStatus?.needs_initial_selection) {
        showDataDirDialog.value = true;
      }
    } catch (error) {
      console.error("Failed to save terms agreement:", error);
    }
  }

  async function handleBrowseDataDir() {
    const selected = await dataDirectory.browseFolder();
    return selected;
  }

  async function handleInitializeDataDir(path: string) {
    await dataDirectory.initialize(path);
    showDataDirDialog.value = false;
  }

  function handleSplashReady() {
    if (isInitializing.value) {
      return;
    }

    showSplash.value = false;

    const settings = settingsStore.settings;
    if (!settings.agreed_to_terms) {
      showTermsDialog.value = true;
    } else if (settingsStore.dataDirStatus?.needs_initial_selection) {
      showDataDirDialog.value = true;
    }

    if (!import.meta.env.DEV) {
      updateStore.checkForUpdateOnStartup();
    }
  }

  return {
    showSplash,
    isInitializing,
    showTermsDialog,
    showDataDirDialog,
    updateStore,
    dataDirectory,
    initializeApp,
    handleAgreeTerms,
    handleBrowseDataDir,
    handleInitializeDataDir,
    handleSplashReady,
    isReady: computed(() => !isInitializing.value),
  };
}
