import { computed } from "vue";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@src/components/workbench/WorkbenchFactGrid.vue";
import { NEXT_SETTINGS_ROUTE_NAME } from "@src/router/pageMeta";
import { useSettingsPageDraft } from "@composables/useSettingsPageDraft";
import { usePaintAppearanceOptions } from "./usePaintAppearanceOptions";
import { usePaintPersonalizationActions } from "./usePaintPersonalizationActions";
import { usePaintSettingsFields } from "./usePaintSettingsFields";

export function usePaintPage() {
  const router = useRouter();
  const fields = usePaintSettingsFields();
  const appearanceOptions = usePaintAppearanceOptions();
  const settingsDraft = useSettingsPageDraft({
    changedGroups: ["Appearance", "Console"],
    syncLocalValues: fields.syncLocalValues,
    prepareForSave: fields.prepareForSave,
  });

  const settings = settingsDraft.settings;
  const {
    exportBusy,
    importBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
  } = usePaintPersonalizationActions({
    settingsDraft,
  });

  const summaryFacts = computed<WorkbenchFactItem[]>(() => [
    {
      label: i18n.t("settings.color_theme"),
      value: appearanceOptions.isThemeProviderActive.value
        ? i18n.t("settings.paint.theme_provider_fact", {
            plugin: appearanceOptions.themeProviderPluginName.value,
          })
        : settings.value
          ? i18n.t(`settings.color_options.${settings.value.color}.label`)
          : "-",
    },
    {
      label: i18n.t("settings.app_display_name"),
      value: settings.value?.app_display_name?.trim() || i18n.t("common.app_name"),
    },
    {
      label: i18n.t("settings.text_customization"),
      value:
        settings.value?.text_color_overrides.title ||
        settings.value?.text_color_overrides.text ||
        settings.value?.text_color_overrides.description
          ? i18n.t("common.enabled")
          : i18n.t("common.default"),
    },
    {
      label: i18n.t("settings.console"),
      value: `${fields.consoleFontSize.value}px / ${fields.maxLogLines.value}`,
    },
  ]);

  function markChanged() {
    settingsDraft.markChanged();
  }

  function openAppearanceSettings(): void {
    void router.push({ name: NEXT_SETTINGS_ROUTE_NAME, hash: "#appearance" });
  }
  return {
    fields,
    appearanceOptions,
    settingsDraft,
    settings,
    exportBusy,
    importBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
    summaryFacts,
    markChanged,
    openAppearanceSettings,
  };
}
