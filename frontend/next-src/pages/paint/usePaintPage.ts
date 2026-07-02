import { computed } from "vue";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@next-src/components/workbench/WorkbenchFactGrid.vue";
import { NEXT_SETTINGS_ROUTE_NAME } from "@next-src/router/pageMeta";
import { useSettingsPageDraft } from "@composables/useSettingsPageDraft";
import { usePaintAppearanceOptions } from "@components/views/paint/usePaintAppearanceOptions";
import { usePaintPersonalizationActions } from "@components/views/paint/usePaintPersonalizationActions";
import { usePaintSettingsFields } from "@components/views/paint/usePaintSettingsFields";

export function usePaintPage() {
  const router = useRouter();
  const fields = usePaintSettingsFields();
  const appearanceOptions = usePaintAppearanceOptions();
  const settingsDraft = useSettingsPageDraft({
    changedGroups: ["Appearance", "Console"],
    syncLocalValues: fields.syncLocalValues,
    prepareForSave: fields.prepareForSave,
    emptyImportMessage: () => i18n.t("settings.no_json"),
  });

  const settings = settingsDraft.settings;
  const {
    personalizationBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
  } = usePaintPersonalizationActions({
    settings,
    settingsDraft,
    markChanged: settingsDraft.markChanged,
  });

  const windowEffectSummary = computed(() => {
    const effect = settings.value?.window_effect || "off";
    const key = `settings.window_effect_help.${effect}`;
    return i18n.t(key);
  });

  const summaryFacts = computed<WorkbenchFactItem[]>(() => [
    {
      label: i18n.t("settings.appearance"),
      value: settings.value ? i18n.t(`settings.theme_options.${settings.value.theme}`) : "-",
    },
    {
      label: i18n.t("settings.font_family"),
      value:
        appearanceOptions.fontFamilyOptions.value.find(
          (item) => item.value === (settings.value?.font_family ?? ""),
        )?.label ?? i18n.t("settings.font_family_default"),
    },
    {
      label: i18n.t("settings.window_effect"),
      value:
        appearanceOptions.windowEffectOptions.value.find(
          (item) => item.value === (settings.value?.window_effect ?? "off"),
        )?.label ?? i18n.t("settings.window_effect_options.off"),
      detail: windowEffectSummary.value,
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

  async function resetSettings() {
    await settingsDraft.resetSettings();
  }

  async function handleImport(json: string) {
    await settingsDraft.importSettings(json);
  }

  return {
    fields,
    appearanceOptions,
    settingsDraft,
    settings,
    personalizationBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
    summaryFacts,
    windowEffectSummary,
    markChanged,
    openAppearanceSettings,
    resetSettings,
    handleImport,
  };
}
