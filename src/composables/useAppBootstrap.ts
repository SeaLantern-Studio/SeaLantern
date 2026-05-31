import { computed, ref } from "vue";
import { useUpdateStore } from "@stores/updateStore";
import { useI18nStore } from "@stores/i18nStore";
import {
  useSettingsStore,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";
import { usePluginStore } from "@stores/pluginStore";
import { useServerStore } from "@stores/serverStore";
import {
  applyTheme,
  applyFontSize,
  applyFontFamily,
  applyMinimalMode,
  applyDeveloperMode,
} from "@utils/theme";

export function useAppBootstrap() {
  const showSplash = ref(true);
  const isInitializing = ref(true);
  const showTermsDialog = ref(false);
  const updateStore = useUpdateStore();
  const i18nStore = useI18nStore();
  const settingsStore = useSettingsStore();
  const pluginStore = usePluginStore();
  const serverStore = useServerStore();

  function applySettingsSideEffects() {
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");
    applyMinimalMode(settings.minimal_mode || false);
    applyDeveloperMode(settings.developer_mode || false);
  }

  async function initializeApp() {
    try {
      await settingsStore.loadSettings();
      await i18nStore.loadLanguageSetting();
      applySettingsSideEffects();

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
    } catch (error) {
      console.error("Failed to save terms agreement:", error);
    }
  }

  function handleSplashReady() {
    if (isInitializing.value) {
      return;
    }

    showSplash.value = false;

    const settings = settingsStore.settings;
    if (!settings.agreed_to_terms) {
      showTermsDialog.value = true;
    }

    if (!import.meta.env.DEV) {
      updateStore.checkForUpdateOnStartup();
    }
  }

  function handleSettingsUpdate(event: CustomEvent<SettingsUpdateEvent>) {
    applyDeveloperMode(event.detail.settings.developer_mode || false);
  }

  function mountSettingsListener() {
    window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);
    return () => {
      window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);
    };
  }

  return {
    showSplash,
    isInitializing,
    showTermsDialog,
    updateStore,
    initializeApp,
    handleAgreeTerms,
    handleSplashReady,
    mountSettingsListener,
    isReady: computed(() => !isInitializing.value),
  };
}
