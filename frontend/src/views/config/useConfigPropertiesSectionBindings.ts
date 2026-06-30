import { computed, type ComputedRef } from "vue";
import type { useConfigCompare } from "@views/config/useConfigCompare";
import type { useConfigPropertiesEditor } from "@views/config/useConfigPropertiesEditor";
import type {
  ConfigEntry as ConfigEntryType,
  DiscoveredServerConfigFile,
  ServerConfigDiscoveryOptions,
  ServerConfigJsonMode,
  ServerConfigSearchHit,
  ServerConfigSearchMode,
  ServerConfigSearchScope,
} from "@api/config";

type PropertiesEditorState = ReturnType<typeof useConfigPropertiesEditor>;
type CompareState = ReturnType<typeof useConfigCompare>;

interface Option {
  label: string;
  value: string | number;
}

interface UseConfigPropertiesSectionBindingsOptions {
  propertiesEditor: PropertiesEditorState;
  compare: CompareState;
  allConfigFiles: ComputedRef<DiscoveredServerConfigFile[]>;
  configFiles: ComputedRef<DiscoveredServerConfigFile[]>;
  selectedConfigLocator: ComputedRef<string>;
  discoveryOptions: ComputedRef<ServerConfigDiscoveryOptions>;
  configSearchQuery: ComputedRef<string>;
  configSearchMode: ComputedRef<ServerConfigSearchMode>;
  configSearchScope: ComputedRef<ServerConfigSearchScope>;
  configSearchResults: ComputedRef<ServerConfigSearchHit[]>;
  configSearchLoading: ComputedRef<boolean>;
  configSearchError: ComputedRef<string | null>;
  updateSelectedConfigFile: (value: string | number) => void;
  updateConfigSearchQuery: (query: string) => void;
  updateConfigSearchMode: (value: string | number) => void;
  updateConfigSearchScope: (value: string | number) => void;
  updateConfigJsonMode: (value: ServerConfigJsonMode) => void;
  importConfigDirectory: () => Promise<void>;
  importConfigFile: () => Promise<void>;
  removeConfigImportDirectory: (path: string) => Promise<void>;
  removeConfigImportFile: (path: string) => Promise<void>;
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
    isPropertiesFile: options.propertiesEditor.isPropertiesFile.value,
    loading: options.propertiesEditor.loading.value,
    compareLoading: options.compare.compareLoading.value,
    compareMode: options.compare.compareMode.value,
    compareSupported: options.compare.compareSupported.value,
    hasCompareTargets: options.compare.hasCompareTargets.value,
    compareTargetServerId: options.compare.compareTargetServerId.value,
    compareServerOptions: options.compare.compareServerOptions.value,
    compareDifferenceBadgeText: options.compare.compareDifferenceBadgeText.value,
    comparePanelRows: options.compare.comparePanelRows.value,
    sourceServerName: options.currentServerName.value,
    targetServerName: options.compareTargetServerName.value,
    hasDiscoveredConfigFiles: options.allConfigFiles.value.length > 0,
    configFiles: options.configFiles.value,
    selectedConfigLocator: options.selectedConfigLocator.value,
    manualImportDirs: options.discoveryOptions.value.manual_import_dirs,
    manualImportFiles: options.discoveryOptions.value.manual_import_files,
    configJsonMode: options.discoveryOptions.value.json_mode,
    configSearchQuery: options.configSearchQuery.value,
    configSearchMode: options.configSearchMode.value,
    configSearchScope: options.configSearchScope.value,
    configSearchResults: options.configSearchResults.value,
    configSearchLoading: options.configSearchLoading.value,
    configSearchError: options.configSearchError.value,
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
    updateSelectedConfigFile: options.updateSelectedConfigFile,
    importConfigDirectory: options.importConfigDirectory,
    importConfigFile: options.importConfigFile,
    removeConfigImportDirectory: options.removeConfigImportDirectory,
    removeConfigImportFile: options.removeConfigImportFile,
    updateConfigSearchQuery: options.updateConfigSearchQuery,
    updateConfigSearchMode: options.updateConfigSearchMode,
    updateConfigSearchScope: options.updateConfigSearchScope,
    updateConfigJsonMode: (value: string | number) =>
      options.updateConfigJsonMode(String(value) as ServerConfigJsonMode),
    updateCompareMode: options.compare.handleCompareModeChange,
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
