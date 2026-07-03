import { computed, shallowRef } from "vue";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@src/components/workbench/WorkbenchFactGrid.vue";
import { NEXT_SETTINGS_ROUTE_NAME } from "@src/router/pageMeta";
import { useSettingsPageDraft } from "@composables/useSettingsPageDraft";
import { usePaintAppearanceOptions } from "./usePaintAppearanceOptions";
import { usePaintPersonalizationActions } from "./usePaintPersonalizationActions";
import { usePaintSettingsFields } from "./usePaintSettingsFields";

export type PaintSectionId = "appearance-theme" | "text-title" | "display-control";

export interface PaintSectionItem {
  id: PaintSectionId;
  label: string;
  description: string;
}

export function usePaintPage() {
  const router = useRouter();
  const fields = usePaintSettingsFields();
  const appearanceOptions = usePaintAppearanceOptions();
  const activeSectionId = shallowRef<PaintSectionId>("appearance-theme");
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

  const sectionItems = computed<readonly PaintSectionItem[]>(() => [
    {
      id: "appearance-theme",
      label: i18n.t("settings.paint.sections.appearance_theme.label"),
      description: i18n.t("settings.paint.sections.appearance_theme.description"),
    },
    {
      id: "text-title",
      label: i18n.t("settings.paint.sections.text_title.label"),
      description: i18n.t("settings.paint.sections.text_title.description"),
    },
    {
      id: "display-control",
      label: i18n.t("settings.paint.sections.display_control.label"),
      description: i18n.t("settings.paint.sections.display_control.description"),
    },
  ]);

  const currentSection = computed(() => {
    return sectionItems.value.find((item) => item.id === activeSectionId.value) ?? sectionItems.value[0];
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

  function selectSection(sectionId: string): void {
    if (
      sectionId === "appearance-theme" ||
      sectionId === "text-title" ||
      sectionId === "display-control"
    ) {
      activeSectionId.value = sectionId;
    }
  }

  return {
    activeSectionId,
    currentSection,
    fields,
    appearanceOptions,
    settingsDraft,
    settings,
    exportBusy,
    importBusy,
    exportPersonalizationPackage,
    importPersonalizationPackage,
    sectionItems,
    summaryFacts,
    markChanged,
    openAppearanceSettings,
    selectSection,
  };
}
