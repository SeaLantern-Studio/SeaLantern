import { computed, type ComputedRef } from "vue";
import type { useConfigCompare } from "@views/config/useConfigCompare";
import type { useConfigPropertiesEditor } from "@views/config/useConfigPropertiesEditor";
import type { ConfigEntry as ConfigEntryType } from "@api/config";

type PropertiesEditorState = ReturnType<typeof useConfigPropertiesEditor>;
type CompareState = ReturnType<typeof useConfigCompare>;

interface Option {
  label: string;
  value: string | number;
}

interface UseConfigPropertiesSectionBindingsOptions {
  propertiesEditor: PropertiesEditorState;
  compare: CompareState;
  currentServerName: ComputedRef<string>;
  compareTargetServerName: ComputedRef<string>;
  translatedDescriptionByKey: ComputedRef<Record<string, string>>;
  gamemodeOptions: ComputedRef<Option[]>;
  difficultyOptions: ComputedRef<Option[]>;
}

export function useConfigPropertiesSectionBindings(
  options: UseConfigPropertiesSectionBindingsOptions,
) {
  const sectionProps = computed(() => ({
    editorMode: options.propertiesEditor.editorMode.value,
    loading: options.propertiesEditor.loading.value,
    compareLoading: options.compare.compareLoading.value,
    compareMode: options.compare.compareMode.value,
    hasCompareTargets: options.compare.hasCompareTargets.value,
    compareTargetServerId: options.compare.compareTargetServerId.value,
    compareServerOptions: options.compare.compareServerOptions.value,
    compareDifferenceBadgeText: options.compare.compareDifferenceBadgeText.value,
    comparePanelRows: options.compare.comparePanelRows.value,
    sourceServerName: options.currentServerName.value,
    targetServerName: options.compareTargetServerName.value,
    categories: options.propertiesEditor.categories.value,
    activeCategory: options.propertiesEditor.activeCategory.value,
    searchQuery: options.propertiesEditor.searchQuery.value,
    filteredEntries: options.propertiesEditor.filteredEntries.value as ConfigEntryType[],
    translatedDescriptionByKey: options.translatedDescriptionByKey.value,
    editValues: options.propertiesEditor.editValues.value,
    numericFieldErrors: options.propertiesEditor.numericFieldErrors.value,
    gamemodeOptions: options.gamemodeOptions.value,
    difficultyOptions: options.difficultyOptions.value,
    sourceDraftText: options.propertiesEditor.sourceDraftText.value,
    compareTargetSourceDraftText: options.compare.compareTargetSourceDraftText.value,
    sourceParseError: options.propertiesEditor.sourceParseError.value,
    hasUnsavedChanges: options.propertiesEditor.hasUnsavedChanges.value,
    saveStatusText: options.propertiesEditor.saveStatusText.value,
    saving: options.propertiesEditor.saving.value,
    reloadCurrentTooltipText: options.propertiesEditor.reloadCurrentTooltipText.value,
    reloadCompareTooltipText: options.propertiesEditor.reloadCompareTooltipText.value,
  }));

  const sectionHandlers = {
    updateCategory: options.propertiesEditor.handleCategoryChange,
    updateSearch: options.propertiesEditor.handleSearchUpdate,
    updateSourceDraft: options.propertiesEditor.updateSourceDraft,
    updateCompareTargetSourceDraft: options.propertiesEditor.updateCompareTargetSourceDraft,
    updateValue: (payload: { key: string; value: string | boolean | number }) =>
      options.propertiesEditor.updateValue(payload.key, payload.value),
    updateCompareTargetValue: (payload: { key: string; value: string | boolean | number }) =>
      options.compare.updateCompareTargetValue(payload.key, payload.value),
    addSourceValue: (payload: { key: string; value: string | boolean | number }) =>
      options.propertiesEditor.updateValue(payload.key, payload.value),
    addTargetValue: (payload: { key: string; value: string | boolean | number }) =>
      options.compare.updateCompareTargetValue(payload.key, payload.value),
    updateCompareTargetServer: options.compare.handleCompareTargetServerChange,
    reloadCurrent: options.propertiesEditor.reloadPropertiesWithGuard,
    reloadCompare: options.propertiesEditor.reloadComparePropertiesWithGuard,
    saveProperties: options.propertiesEditor.saveProperties,
  };

  return {
    sectionProps,
    sectionHandlers,
  };
}
