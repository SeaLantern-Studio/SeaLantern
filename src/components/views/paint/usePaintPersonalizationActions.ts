import { ref, type Ref } from "vue";
import type { AppSettings } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useToast } from "@composables/useToast";
import { usePluginStore } from "@stores/pluginStore";
import { useSettingsStore } from "@stores/settingsStore";
import type { useSettingsPageDraft } from "@composables/useSettingsPageDraft";

type SettingsDraftState = ReturnType<typeof useSettingsPageDraft>;

interface UsePaintPersonalizationActionsOptions {
  settings: Ref<AppSettings | null>;
  settingsDraft: SettingsDraftState;
  markChanged: () => void;
}

export function usePaintPersonalizationActions(options: UsePaintPersonalizationActionsOptions) {
  const pluginStore = usePluginStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();
  const personalizationBusy = ref(false);

  async function exportPersonalizationPackage() {
    personalizationBusy.value = true;
    options.settingsDraft.clearError();
    try {
      const suggestedName = await settingsStore.getPersonalizationPackageSuggestedName();
      const filePath = await systemApi.pickPersonalizationExportFile(suggestedName);
      if (!filePath) {
        return;
      }

      await settingsStore.exportPersonalizationPackage(filePath);
      toast.success(i18n.t("settings.personalization_export_success"));
    } catch (error) {
      const message = String(error);
      options.settingsDraft.setError(message);
      toast.error(message);
    } finally {
      personalizationBusy.value = false;
    }
  }

  async function importPersonalizationPackage() {
    personalizationBusy.value = true;
    options.settingsDraft.clearError();
    try {
      const filePath = await systemApi.pickPersonalizationImportFile();
      if (!filePath) {
        return;
      }

      const result = await settingsStore.importPersonalizationPackage(filePath);
      options.settingsDraft.applyStoreSnapshot(result.settings);
      await pluginStore.injectAllPluginCss();

      if (result.skipped_plugins.length > 0) {
        toast.warning(i18n.t("settings.personalization_import_partial"));
      } else {
        toast.success(i18n.t("settings.personalization_import_success"));
      }
    } catch (error) {
      const message = String(error);
      options.settingsDraft.setError(message);
      toast.error(message);
    } finally {
      personalizationBusy.value = false;
    }
  }

  async function pickBackgroundImage() {
    try {
      const result = await systemApi.pickImageFile();
      console.log("Selected image:", result);
      if (result && options.settings.value) {
        options.settings.value.background_image = result;
        options.markChanged();
      }
    } catch (error) {
      console.error("Pick image error:", error);
      options.settingsDraft.setError(String(error));
    }
  }

  function clearBackgroundImage() {
    if (!options.settings.value) {
      return;
    }

    options.settings.value.background_image = "";
    options.markChanged();
  }

  return {
    personalizationBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
    pickBackgroundImage,
    clearBackgroundImage,
  };
}
