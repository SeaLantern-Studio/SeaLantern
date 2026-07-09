import { ref } from "vue";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useToast } from "@composables/useToast";
import { usePluginStore } from "@stores/pluginStore";
import { useSettingsStore } from "@stores/settingsStore";
import type { useSettingsPageDraft } from "@composables/useSettingsPageDraft";

type SettingsDraftState = ReturnType<typeof useSettingsPageDraft>;

interface UsePaintPersonalizationActionsOptions {
  settingsDraft: SettingsDraftState;
}

export function usePaintPersonalizationActions(options: UsePaintPersonalizationActionsOptions) {
  const pluginStore = usePluginStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();
  const exportBusy = ref(false);
  const importBusy = ref(false);

  async function exportPersonalizationPackage() {
    if (exportBusy.value) {
      return;
    }

    exportBusy.value = true;
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
      exportBusy.value = false;
    }
  }

  async function importPersonalizationPackage() {
    if (importBusy.value) {
      return;
    }

    importBusy.value = true;
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
        toast.warning(
          i18n.t("settings.personalization_import_partial", {
            count: result.skipped_plugins.length,
          }),
        );
      } else {
        toast.success(i18n.t("settings.personalization_import_success"));
      }
    } catch (error) {
      const message = String(error);
      options.settingsDraft.setError(message);
      toast.error(message);
    } finally {
      importBusy.value = false;
    }
  }

  return {
    exportBusy,
    importBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
  };
}
