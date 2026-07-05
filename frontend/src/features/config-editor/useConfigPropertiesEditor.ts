import { computed, ref, shallowRef, type ComputedRef, type Ref } from "vue";
import { configApi } from "@api/config";
import type {
  ConfigEntry as ConfigEntryType,
  DiscoveredServerConfigFile,
  ServerConfigDiscoveryOptions,
  ServerConfigFileKind,
} from "@api/config";
import { i18n } from "@language";
import { useConfigPropertiesModeSwitch } from "@src/features/config-editor/useConfigPropertiesModeSwitch";
import { useConfigPropertiesReloadGuard } from "@src/features/config-editor/useConfigPropertiesReloadGuard";
import { useConfigPropertiesSaveFlow } from "@src/features/config-editor/useConfigPropertiesSaveFlow";

interface CompareContext {
  compareMode: Ref<boolean>;
  compareTargetServerId: Ref<string>;
  compareTargetEntries: Ref<ConfigEntryType[]>;
  compareTargetPath: Ref<string>;
  compareTargetServerName: ComputedRef<string>;
  compareTargetServerPropertiesPath: Ref<string>;
  compareTargetDraftValues: Ref<Record<string, string>>;
  compareTargetLoadedValues: Ref<Record<string, string>>;
  compareTargetSourceDraftText: Ref<string>;
  compareTargetLoadedSourceText: Ref<string>;
  compareTargetNumericFieldErrors: Ref<Record<string, string>>;
  loadCompareProperties: () => Promise<void>;
  applyParsedCompareTargetState: (sourceText: string) => Promise<void>;
  applyCompareTargetSourceDraftToVisualState: (sourceText: string) => Promise<void>;
  buildCompareTargetPreviewSource: () => Promise<string>;
  prepareCompareTargetSourceDraftForSourceMode: () => Promise<void>;
  updateCompareTargetSourceDraft: (value: string) => void;
  captureDifferenceCategorySnapshot: () => void;
}

const COMPARE_DIFFERENCE_CATEGORY = "difference";

interface UseConfigPropertiesEditorOptions {
  serverPath: Ref<string>;
  currentConfigFile: Ref<DiscoveredServerConfigFile | null>;
  currentConfigLocator: Ref<string>;
  discoveryOptions: Ref<ServerConfigDiscoveryOptions>;
  selectedConfigRelativePath: Ref<string>;
  currentConfigFilePath: Ref<string>;
  currentConfigLabel: ComputedRef<string>;
  currentServerId: Ref<string | null>;
  currentServerName: ComputedRef<string>;
  setError: (message: string | null) => void;
  setSuccess: (message: string | null) => void;
  updateCurrentServerPort: (port: string) => void;
}

function getTranslatedPropertyDescription(key: string) {
  const translationKey = `config.properties.${key}`;
  const translated = i18n.t(translationKey);
  return translated === translationKey ? "" : translated;
}

export function useConfigPropertiesEditor(options: UseConfigPropertiesEditorOptions) {
  const entries = ref<ConfigEntryType[]>([]);
  const editValues = ref<Record<string, string>>({});
  const loadedValues = ref<Record<string, string>>({});
  const loading = ref(false);
  const saving = ref(false);
  const searchQuery = ref("");
  const activeCategory = ref("all");
  const editorMode = ref<"visual" | "source">("visual");
  const sourceDraftText = ref("");
  const loadedSourceText = ref("");
  const visualModeBaseValues = ref<Record<string, string>>({});
  const sourceParseError = ref<string | null>(null);

  const compareContext = shallowRef<CompareContext | null>(null);
  const currentConfigKind = computed<ServerConfigFileKind | null>(
    () => options.currentConfigFile.value?.kind ?? null,
  );
  const isPropertiesFile = computed(() => currentConfigKind.value === "properties");

  const categories = computed(() => {
    const cats = new Set(entries.value.map((e) => e.category));
    const categoryList = ["all", ...Array.from(cats)];
    if (isPropertiesFile.value && compareContext.value?.compareMode.value) {
      categoryList.push(COMPARE_DIFFERENCE_CATEGORY);
    }
    return categoryList;
  });

  const filteredEntries = computed(() => {
    return entries.value.filter((e: ConfigEntryType) => {
      const matchCat = activeCategory.value === "all" || e.category === activeCategory.value;
      const matchSearch =
        !searchQuery.value ||
        e.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        (e.description ?? "").toLowerCase().includes(searchQuery.value.toLowerCase());
      return matchCat && matchSearch;
    });
  });

  const numericFieldErrors = computed(() => {
    const errors: Record<string, string> = {};

    for (const entry of entries.value) {
      if (entry.value_type !== "number") {
        continue;
      }

      const value = editValues.value[entry.key]?.trim() ?? "";
      if (value.length === 0 || !/^-?\d+$/.test(value)) {
        errors[entry.key] = i18n.t("config.invalid_integer_field", { field: entry.key });
      }
    }

    return errors;
  });

  const hasInvalidNumericValues = computed(() => Object.keys(numericFieldErrors.value).length > 0);

  const hasUnsavedChanges = computed(() => {
    const context = compareContext.value;
    const targetDirty =
      isPropertiesFile.value &&
      !!context?.compareMode.value &&
      (context.compareTargetSourceDraftText.value !== context.compareTargetLoadedSourceText.value ||
        !areMapValuesEqual(
          context.compareTargetDraftValues.value,
          context.compareTargetLoadedValues.value,
        ));

    if (editorMode.value === "source") {
      return sourceDraftText.value !== loadedSourceText.value || targetDirty;
    }

    const sourceDirty = sourceDraftText.value !== loadedSourceText.value;
    const visualDirty = !areMapValuesEqual(editValues.value, loadedValues.value);
    if (!context?.compareMode.value) {
      return sourceDirty || visualDirty;
    }

    return sourceDirty || visualDirty || targetDirty;
  });

  const saveStatusText = computed(() =>
    hasUnsavedChanges.value ? i18n.t("config.status_unsaved") : i18n.t("config.status_loaded"),
  );

  const reloadCurrentTooltipText = computed(() =>
    i18n.t("config.reload_properties_tooltip", {
      name: `${options.currentServerName.value || i18n.t("config.current_server")} · ${options.currentConfigLabel.value}`,
    }),
  );

  const reloadCompareTooltipText = computed(() => {
    const context = compareContext.value;
    return i18n.t("config.reload_properties_tooltip", {
      name: context?.compareTargetServerName.value || i18n.t("config.compare.target_server"),
    });
  });

  function bindCompareContext(context: CompareContext) {
    compareContext.value = context;
  }

  async function applyParsedSourceState(
    sourceText: string,
    targetMode: "visual" | "source" = "visual",
  ) {
    if (!isPropertiesFile.value) {
      entries.value = [];
      editValues.value = {};
      loadedValues.value = {};
      visualModeBaseValues.value = {};
      sourceDraftText.value = sourceText;
      loadedSourceText.value = sourceText;
      modeSwitch.clearSourceParseError();
      modeSwitch.visualDraftDirty.value = false;
      editorMode.value = "source";
      return null;
    }

    const parsed = await configApi.parseServerPropertiesSource(sourceText);
    entries.value = parsed.entries as ConfigEntryType[];
    editValues.value = { ...parsed.raw };
    loadedValues.value = { ...parsed.raw };
    visualModeBaseValues.value = { ...parsed.raw };
    sourceDraftText.value = sourceText;
    loadedSourceText.value = sourceText;
    modeSwitch.clearSourceParseError();
    modeSwitch.visualDraftDirty.value = false;
    editorMode.value = targetMode;

    const port = parsed.raw["server-port"];
    if (port) {
      options.updateCurrentServerPort(port);
    }

    return parsed;
  }

  const saveFlow = useConfigPropertiesSaveFlow({
    serverPath: options.serverPath,
    currentConfigRelativePath: computed(() => options.currentConfigFile.value?.relative_path ?? ""),
    currentConfigLocator: options.currentConfigLocator,
    currentConfigFilePath: options.currentConfigFilePath,
    currentConfigKind,
    discoveryOptions: options.discoveryOptions,
    currentServerId: options.currentServerId,
    currentServerName: options.currentServerName,
    editorMode,
    sourceDraftText,
    loadedSourceText,
    editValues,
    loadedValues,
    visualModeBaseValues,
    saving,
    hasUnsavedChanges,
    numericFieldErrors,
    setError: options.setError,
    setSuccess: options.setSuccess,
    getCompareContext: () => compareContext.value,
    applyParsedSourceState,
  });

  const modeSwitch = useConfigPropertiesModeSwitch({
    serverPath: options.serverPath,
    currentConfigKind,
    editorMode,
    sourceDraftText,
    editValues,
    visualModeBaseValues,
    sourceParseError,
    setError: options.setError,
    buildVisualPreviewSource: saveFlow.buildVisualPreviewSource,
    getCompareContext: () => compareContext.value,
  });

  async function loadProperties() {
    if (!options.serverPath.value || !options.currentConfigFile.value) return;

    loading.value = true;
    options.setError(null);
    try {
      const sourceText = await configApi.readServerConfigSource(
        options.serverPath.value,
        options.currentConfigFile.value.relative_path,
        options.currentConfigLocator.value || undefined,
        options.discoveryOptions.value,
      );
      await applyParsedSourceState(sourceText, isPropertiesFile.value ? "visual" : "source");

      const context = compareContext.value;
      if (
        isPropertiesFile.value &&
        context?.compareMode.value &&
        context.compareTargetServerId.value
      ) {
        await context.loadCompareProperties();
      }
    } catch (e) {
      options.setError(String(e));
      entries.value = [];
      editValues.value = {};
      loadedValues.value = {};
      sourceDraftText.value = "";
      loadedSourceText.value = "";

      const context = compareContext.value;
      if (context) {
        context.compareTargetEntries.value = [];
        context.compareTargetDraftValues.value = {};
        context.compareTargetLoadedValues.value = {};
        context.compareTargetSourceDraftText.value = "";
        context.compareTargetLoadedSourceText.value = "";
      }
    } finally {
      loading.value = false;
    }
  }

  async function loadCurrentPropertiesOnly() {
    if (!options.serverPath.value || !options.currentConfigFile.value) return;

    loading.value = true;
    options.setError(null);
    try {
      const sourceText = await configApi.readServerConfigSource(
        options.serverPath.value,
        options.currentConfigFile.value.relative_path,
        options.currentConfigLocator.value || undefined,
        options.discoveryOptions.value,
      );
      await applyParsedSourceState(sourceText, isPropertiesFile.value ? "visual" : "source");
    } catch (e) {
      options.setError(String(e));
      entries.value = [];
      editValues.value = {};
      loadedValues.value = {};
      sourceDraftText.value = "";
      loadedSourceText.value = "";
    } finally {
      loading.value = false;
    }
  }

  const reloadGuard = useConfigPropertiesReloadGuard({
    currentServerName: options.currentServerName,
    currentConfigLabel: options.currentConfigLabel,
    sourceDraftText,
    loadedSourceText,
    editValues,
    loadedValues,
    getCompareContext: () => compareContext.value,
    loadCurrentPropertiesOnly,
  });

  function updateValue(key: string, value: string | boolean | number) {
    if (!entries.value.some((entry) => entry.key === key)) {
      const compareEntry = compareContext.value?.compareTargetEntries.value.find(
        (entry) => entry.key === key,
      );
      if (compareEntry) {
        entries.value = [...entries.value, { ...compareEntry }];
      }
    }

    editValues.value[key] = String(value);
    if (isPropertiesFile.value) {
      modeSwitch.markVisualDraftDirty();
    }
  }

  function updateSourceDraft(value: string) {
    sourceDraftText.value = value;
    modeSwitch.clearSourceParseError();
  }

  function updateCompareTargetSourceDraft(value: string) {
    compareContext.value?.updateCompareTargetSourceDraft(value);
    modeSwitch.clearSourceParseError();
  }

  async function handleEditorModeChange(mode: string | null) {
    const parsedEntries = await modeSwitch.handleEditorModeChange(mode);
    if (parsedEntries) {
      entries.value = parsedEntries;
    }
  }

  function handleCategoryChange(category: string) {
    if (
      category === COMPARE_DIFFERENCE_CATEGORY &&
      activeCategory.value !== COMPARE_DIFFERENCE_CATEGORY
    ) {
      compareContext.value?.captureDifferenceCategorySnapshot();
    }

    activeCategory.value = category;
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  function handleSearchUpdate(value: string) {
    searchQuery.value = value;
  }

  function selectConfigFile(relativePath: string | number) {
    options.selectedConfigRelativePath.value = String(relativePath);
  }

  return {
    entries,
    editValues,
    loadedValues,
    loading,
    saving,
    searchQuery,
    activeCategory,
    editorMode,
    sourceDraftText,
    sourceParseError,
    showDiscardConfirm: reloadGuard.showDiscardConfirm,
    pendingReloadSide: reloadGuard.pendingReloadSide,
    showSaveDiffModal: saveFlow.showSaveDiffModal,
    pendingSaveItems: saveFlow.pendingSaveItems,
    categories,
    filteredEntries,
    numericFieldErrors,
    currentConfigKind,
    currentConfigFile: options.currentConfigFile,
    currentConfigLabel: options.currentConfigLabel,
    currentConfigFilePath: options.currentConfigFilePath,
    isPropertiesFile,
    hasUnsavedChanges,
    saveStatusText,
    hasInvalidNumericValues,
    reloadCurrentTooltipText,
    reloadCompareTooltipText,
    discardConfirmTitle: reloadGuard.discardConfirmTitle,
    discardConfirmMessage: reloadGuard.discardConfirmMessage,
    pendingSaveItemsWithStats: saveFlow.pendingSaveItemsWithStats,
    bindCompareContext,
    getTranslatedPropertyDescription,
    loadProperties,
    loadCurrentPropertiesOnly,
    selectConfigFile,
    saveProperties: saveFlow.saveProperties,
    updateValue,
    updateSourceDraft,
    updateCompareTargetSourceDraft,
    handleEditorModeChange,
    confirmSaveProperties: saveFlow.confirmSaveProperties,
    closeSaveDiffModal: saveFlow.closeSaveDiffModal,
    closeDiscardDialog: reloadGuard.closeDiscardDialog,
    reloadPropertiesWithGuard: reloadGuard.reloadPropertiesWithGuard,
    reloadComparePropertiesWithGuard: reloadGuard.reloadComparePropertiesWithGuard,
    confirmReloadDiscard: reloadGuard.confirmReloadDiscard,
    handleCategoryChange,
    handleSearchUpdate,
  };
}

function areMapValuesEqual(a: Record<string, string>, b: Record<string, string>) {
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);

  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (a[key] !== b[key]) {
      return false;
    }
  }

  return true;
}
