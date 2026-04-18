import { computed, ref, shallowRef, type ComputedRef, type Ref } from "vue";
import { configApi } from "@api/config";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { i18n } from "@language";
import { buildDiffLines } from "@utils/configDiff";

export interface PendingSaveItem {
  serverId: string;
  serverName: string;
  serverPath: string;
  filePath: string;
  originalText: string;
  modifiedText: string;
}

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
  serverPropertiesPath: Ref<string>;
  currentServerId: Ref<string | null>;
  currentServerName: ComputedRef<string>;
  setError: (message: string | null) => void;
  setSuccess: (message: string | null) => void;
  updateCurrentServerPort: (port: string) => void;
}

function getChangedValues(
  draftValues: Record<string, string>,
  baseValues: Record<string, string>,
): Record<string, string> {
  const changedValues: Record<string, string> = {};

  for (const [key, value] of Object.entries(draftValues)) {
    if (baseValues[key] !== value) {
      changedValues[key] = value;
    }
  }

  return changedValues;
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
  const visualDraftDirty = ref(false);
  const modeSwitching = ref(false);
  const sourceParseError = ref<string | null>(null);
  const showDiscardConfirm = ref(false);
  const pendingReloadSide = ref<"current" | "compare" | null>(null);
  const showSaveDiffModal = ref(false);
  const pendingSaveItems = ref<PendingSaveItem[]>([]);

  const compareContext = shallowRef<CompareContext | null>(null);

  const categories = computed(() => {
    const cats = new Set(entries.value.map((e) => e.category));
    const categoryList = ["all", ...Array.from(cats)];
    if (compareContext.value?.compareMode.value) {
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
        errors[entry.key] = `${entry.key} 需要填写整数`;
      }
    }

    return errors;
  });

  const hasInvalidNumericValues = computed(() => Object.keys(numericFieldErrors.value).length > 0);

  const hasUnsavedChanges = computed(() => {
    const context = compareContext.value;
    const targetDirty =
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

  const reloadCurrentTooltipText = computed(
    () => `重新载入${options.currentServerName.value || i18n.t("config.current_server")}属性`,
  );

  const reloadCompareTooltipText = computed(() => {
    const context = compareContext.value;
    return `重新载入${context?.compareTargetServerName.value || i18n.t("config.compare.target_server")}属性`;
  });

  const currentSideDirty = computed(
    () =>
      sourceDraftText.value !== loadedSourceText.value ||
      !areMapValuesEqual(editValues.value, loadedValues.value),
  );

  const compareSideDirty = computed(() => {
    const context = compareContext.value;
    if (!context) {
      return false;
    }
    return (
      context.compareTargetSourceDraftText.value !== context.compareTargetLoadedSourceText.value ||
      !areMapValuesEqual(
        context.compareTargetDraftValues.value,
        context.compareTargetLoadedValues.value,
      )
    );
  });

  const discardConfirmTitle = computed(() => {
    if (pendingReloadSide.value === "compare") {
      return "丢弃对照侧修改";
    }
    if (pendingReloadSide.value === "current") {
      return "丢弃当前侧修改";
    }
    return i18n.t("config.discard_title");
  });

  const discardConfirmMessage = computed(() => {
    const context = compareContext.value;
    if (pendingReloadSide.value === "compare") {
      return `重新载入将丢弃 ${context?.compareTargetServerName.value || i18n.t("config.compare.target_server")} 的未保存属性修改。`;
    }
    if (pendingReloadSide.value === "current") {
      return `重新载入将丢弃 ${options.currentServerName.value || i18n.t("config.current_server")} 的未保存属性修改。`;
    }
    return i18n.t("config.discard_message");
  });

  const pendingSaveItemsWithStats = computed(() =>
    pendingSaveItems.value.map((item) => {
      const diffLines = buildDiffLines(item.originalText, item.modifiedText);
      let additions = 0;
      let deletions = 0;

      for (const line of diffLines) {
        if (line.type === "addition") additions += 1;
        if (line.type === "deletion") deletions += 1;
      }

      return {
        ...item,
        additions,
        deletions,
      };
    }),
  );

  function bindCompareContext(context: CompareContext) {
    compareContext.value = context;
  }

  function getTranslatedPropertyDescription(key: string) {
    const translationKey = `config.properties.${key}`;
    const translated = i18n.t(translationKey);
    return translated === translationKey ? "" : translated;
  }

  function getChangedPropertyValues() {
    const baseValues =
      sourceDraftText.value !== loadedSourceText.value
        ? visualModeBaseValues.value
        : loadedValues.value;
    return getChangedValues(editValues.value, baseValues);
  }

  async function buildVisualPreviewSource() {
    const changedValues = getChangedPropertyValues();

    if (sourceDraftText.value !== loadedSourceText.value) {
      return configApi.previewServerPropertiesWriteFromSource(sourceDraftText.value, changedValues);
    }

    return configApi.previewServerPropertiesWrite(options.serverPath.value, changedValues);
  }

  async function applyParsedSourceState(
    sourceText: string,
    targetMode: "visual" | "source" = "visual",
  ) {
    const parsed = await configApi.parseServerPropertiesSource(sourceText);
    entries.value = parsed.entries as ConfigEntryType[];
    editValues.value = { ...parsed.raw };
    loadedValues.value = { ...parsed.raw };
    visualModeBaseValues.value = { ...parsed.raw };
    sourceDraftText.value = sourceText;
    loadedSourceText.value = sourceText;
    sourceParseError.value = null;
    visualDraftDirty.value = false;
    editorMode.value = targetMode;

    const port = parsed.raw["server-port"];
    if (port) {
      options.updateCurrentServerPort(port);
    }

    return parsed;
  }

  async function loadProperties() {
    if (!options.serverPath.value) return;

    loading.value = true;
    options.setError(null);
    try {
      const sourceText = await configApi.readServerPropertiesSource(options.serverPath.value);
      await applyParsedSourceState(sourceText, "visual");

      const context = compareContext.value;
      if (context?.compareMode.value && context.compareTargetServerId.value) {
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
    if (!options.serverPath.value) return;

    loading.value = true;
    options.setError(null);
    try {
      const sourceText = await configApi.readServerPropertiesSource(options.serverPath.value);
      await applyParsedSourceState(sourceText, "visual");
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

  async function saveProperties() {
    if (!options.serverPath.value || !hasUnsavedChanges.value || saving.value) return;

    options.setError(null);
    pendingSaveItems.value = [];

    try {
      if (editorMode.value === "visual" && hasInvalidNumericValues.value) {
        const invalidKeys = Object.keys(numericFieldErrors.value);
        options.setError(`以下字段需要填写整数：${invalidKeys.join("、")}`);
        return;
      }

      const context = compareContext.value;
      if (
        editorMode.value === "visual" &&
        context &&
        context.compareMode.value &&
        Object.keys(context.compareTargetNumericFieldErrors.value).length > 0
      ) {
        const invalidKeys = Object.keys(context.compareTargetNumericFieldErrors.value);
        options.setError(`以下字段需要填写整数：${invalidKeys.join("、")}`);
        return;
      }

      const pendingItems: PendingSaveItem[] = [];

      const sourceChanged =
        sourceDraftText.value !== loadedSourceText.value ||
        !areMapValuesEqual(editValues.value, loadedValues.value);
      if (sourceChanged) {
        const latestSourceText = await configApi.readServerPropertiesSource(
          options.serverPath.value,
        );
        const nextSourceText =
          editorMode.value === "visual" ? await buildVisualPreviewSource() : sourceDraftText.value;

        if (nextSourceText !== latestSourceText) {
          pendingItems.push({
            serverId: options.currentServerId.value || "",
            serverName: options.currentServerName.value || i18n.t("config.compare.source_server"),
            serverPath: options.serverPath.value,
            filePath: options.serverPropertiesPath.value,
            originalText: latestSourceText,
            modifiedText: nextSourceText,
          });
        } else {
          await applyParsedSourceState(latestSourceText, editorMode.value);
        }
      }

      const targetDirty =
        !!context?.compareMode.value &&
        (context.compareTargetSourceDraftText.value !==
          context.compareTargetLoadedSourceText.value ||
          !areMapValuesEqual(
            context.compareTargetDraftValues.value,
            context.compareTargetLoadedValues.value,
          ));
      if (targetDirty && context?.compareTargetPath.value) {
        const latestTargetText = await configApi.readServerPropertiesSource(
          context.compareTargetPath.value,
        );
        const nextTargetText =
          editorMode.value === "visual"
            ? await context.buildCompareTargetPreviewSource()
            : context.compareTargetSourceDraftText.value;

        if (nextTargetText !== latestTargetText) {
          pendingItems.push({
            serverId: context.compareTargetServerId.value,
            serverName:
              context.compareTargetServerName.value || i18n.t("config.compare.target_server"),
            serverPath: context.compareTargetPath.value,
            filePath: context.compareTargetServerPropertiesPath.value,
            originalText: latestTargetText,
            modifiedText: nextTargetText,
          });
        } else {
          await context.applyParsedCompareTargetState(latestTargetText);
        }
      }

      pendingSaveItems.value = pendingItems;

      if (pendingSaveItems.value.length === 0) {
        options.setSuccess(i18n.t("config.no_changes_to_save"));
        setTimeout(() => options.setSuccess(null), 3000);
        return;
      }

      showSaveDiffModal.value = true;
    } catch (e) {
      options.setError(String(e));
    }
  }

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
    visualDraftDirty.value = true;
  }

  function updateSourceDraft(value: string) {
    sourceDraftText.value = value;
    sourceParseError.value = null;
  }

  function updateCompareTargetSourceDraft(value: string) {
    compareContext.value?.updateCompareTargetSourceDraft(value);
    sourceParseError.value = null;
  }

  async function handleEditorModeChange(mode: string | null) {
    const targetMode = mode === "source" ? "source" : "visual";
    if (targetMode === editorMode.value || modeSwitching.value || !options.serverPath.value) return;

    modeSwitching.value = true;
    options.setError(null);

    try {
      if (targetMode === "source") {
        if (visualDraftDirty.value) {
          sourceDraftText.value = await buildVisualPreviewSource();
          visualDraftDirty.value = false;
        }
        if (compareContext.value?.compareMode.value) {
          await compareContext.value.prepareCompareTargetSourceDraftForSourceMode();
        }
        sourceParseError.value = null;
        editorMode.value = "source";
        return;
      }

      const parsed = await configApi.parseServerPropertiesSource(sourceDraftText.value);
      entries.value = parsed.entries as ConfigEntryType[];
      editValues.value = { ...parsed.raw };
      visualModeBaseValues.value = { ...parsed.raw };
      const context = compareContext.value;
      if (context?.compareMode.value) {
        await context.applyCompareTargetSourceDraftToVisualState(
          context.compareTargetSourceDraftText.value,
        );
      }
      visualDraftDirty.value = false;
      sourceParseError.value = null;
      editorMode.value = "visual";
    } catch (e) {
      sourceParseError.value = i18n.t("config.source_parse_failed");
      options.setError(String(e));
    } finally {
      modeSwitching.value = false;
    }
  }

  async function confirmSaveProperties() {
    if (pendingSaveItems.value.length === 0 || saving.value) return;

    saving.value = true;
    options.setError(null);
    options.setSuccess(null);

    try {
      await Promise.all(
        pendingSaveItems.value.map((item) =>
          configApi.writeServerPropertiesSource(item.serverPath, item.modifiedText),
        ),
      );

      const savedCurrent = pendingSaveItems.value.find(
        (item) => item.serverId === options.currentServerId.value,
      );
      if (savedCurrent) {
        await applyParsedSourceState(savedCurrent.modifiedText, editorMode.value);
      }

      const context = compareContext.value;
      const savedTarget = pendingSaveItems.value.find(
        (item) => item.serverId === context?.compareTargetServerId.value,
      );
      if (savedTarget && context) {
        await context.applyParsedCompareTargetState(savedTarget.modifiedText);
      }

      pendingSaveItems.value = [];
      options.setSuccess(i18n.t("config.saved"));
      showSaveDiffModal.value = false;
      setTimeout(() => options.setSuccess(null), 3000);
    } catch (e) {
      options.setError(String(e));
    } finally {
      saving.value = false;
    }
  }

  function closeSaveDiffModal() {
    if (saving.value) return;
    pendingSaveItems.value = [];
    showSaveDiffModal.value = false;
  }

  async function reloadPropertiesWithGuard() {
    pendingReloadSide.value = "current";
    if (currentSideDirty.value) {
      showDiscardConfirm.value = true;
      return;
    }

    await loadCurrentPropertiesOnly();
    pendingReloadSide.value = null;
  }

  async function reloadComparePropertiesWithGuard() {
    const context = compareContext.value;
    if (!context?.compareMode.value) {
      return;
    }

    pendingReloadSide.value = "compare";
    if (compareSideDirty.value) {
      showDiscardConfirm.value = true;
      return;
    }

    await context.loadCompareProperties();
    pendingReloadSide.value = null;
  }

  async function confirmReloadDiscard() {
    showDiscardConfirm.value = false;
    const context = compareContext.value;
    if (pendingReloadSide.value === "compare" && context) {
      await context.loadCompareProperties();
    } else {
      await loadCurrentPropertiesOnly();
    }
    pendingReloadSide.value = null;
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
    showDiscardConfirm,
    pendingReloadSide,
    showSaveDiffModal,
    pendingSaveItems,
    categories,
    filteredEntries,
    numericFieldErrors,
    hasUnsavedChanges,
    saveStatusText,
    hasInvalidNumericValues,
    reloadCurrentTooltipText,
    reloadCompareTooltipText,
    discardConfirmTitle,
    discardConfirmMessage,
    pendingSaveItemsWithStats,
    bindCompareContext,
    getTranslatedPropertyDescription,
    loadProperties,
    loadCurrentPropertiesOnly,
    saveProperties,
    updateValue,
    updateSourceDraft,
    updateCompareTargetSourceDraft,
    handleEditorModeChange,
    confirmSaveProperties,
    closeSaveDiffModal,
    reloadPropertiesWithGuard,
    reloadComparePropertiesWithGuard,
    confirmReloadDiscard,
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
