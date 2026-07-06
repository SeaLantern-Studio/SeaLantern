import { computed, ref, type ComputedRef, type Ref } from "vue";
import { configApi } from "@api/config";
import type { ServerConfigDiscoveryOptions, ServerConfigFileKind } from "@api/config";
import { i18n } from "@language";
import { buildDiffLines } from "@utils/configDiff";

export interface PendingSaveItem {
  serverId: string;
  serverName: string;
  serverPath: string;
  filePath: string;
  relativePath: string;
  locator?: string;
  discoveryOptions?: ServerConfigDiscoveryOptions;
  originalText: string;
  modifiedText: string;
}

interface SaveCompareContext {
  compareMode: Ref<boolean>;
  compareTargetServerId: Ref<string>;
  compareTargetPath: Ref<string>;
  compareTargetServerName: ComputedRef<string>;
  compareTargetServerPropertiesPath: Ref<string>;
  compareTargetDraftValues: Ref<Record<string, string>>;
  compareTargetLoadedValues: Ref<Record<string, string>>;
  compareTargetSourceDraftText: Ref<string>;
  compareTargetLoadedSourceText: Ref<string>;
  compareTargetNumericFieldErrors: Ref<Record<string, string>>;
  buildCompareTargetPreviewSource: () => Promise<string>;
  applyParsedCompareTargetState: (sourceText: string) => Promise<void>;
}

interface UseConfigPropertiesSaveFlowOptions {
  serverPath: Ref<string>;
  currentConfigRelativePath: Ref<string>;
  currentConfigLocator: Ref<string>;
  currentConfigFilePath: Ref<string>;
  currentConfigKind: Ref<ServerConfigFileKind | null>;
  discoveryOptions: Ref<ServerConfigDiscoveryOptions>;
  currentServerId: Ref<string | null>;
  currentServerName: ComputedRef<string>;
  editorMode: Ref<"visual" | "source">;
  sourceDraftText: Ref<string>;
  loadedSourceText: Ref<string>;
  editValues: Ref<Record<string, string>>;
  loadedValues: Ref<Record<string, string>>;
  visualModeBaseValues: Ref<Record<string, string>>;
  saving: Ref<boolean>;
  hasUnsavedChanges: ComputedRef<boolean>;
  numericFieldErrors: ComputedRef<Record<string, string>>;
  setError: (message: string | null) => void;
  setSuccess: (message: string | null) => void;
  getCompareContext: () => SaveCompareContext | null;
  applyParsedSourceState: (
    sourceText: string,
    targetMode?: "visual" | "source",
  ) => Promise<unknown>;
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

export function useConfigPropertiesSaveFlow(options: UseConfigPropertiesSaveFlowOptions) {
  const showSaveDiffModal = ref(false);
  const pendingSaveItems = ref<PendingSaveItem[]>([]);

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

  function getChangedPropertyValues() {
    const baseValues =
      options.sourceDraftText.value !== options.loadedSourceText.value
        ? options.visualModeBaseValues.value
        : options.loadedValues.value;
    return getChangedValues(options.editValues.value, baseValues);
  }

  async function buildVisualPreviewSource() {
    if (options.currentConfigKind.value !== "properties") {
      return options.sourceDraftText.value;
    }

    const changedValues = getChangedPropertyValues();
    const baseSource =
      options.sourceDraftText.value !== options.loadedSourceText.value
        ? options.sourceDraftText.value
        : options.loadedSourceText.value;
    return configApi.previewServerPropertiesWriteFromSource(baseSource, changedValues);
  }

  async function saveProperties() {
    if (!options.serverPath.value || !options.hasUnsavedChanges.value || options.saving.value) {
      return;
    }

    options.setError(null);
    pendingSaveItems.value = [];

    try {
      if (
        options.editorMode.value === "visual" &&
        Object.keys(options.numericFieldErrors.value).length > 0
      ) {
        const invalidKeys = Object.keys(options.numericFieldErrors.value);
        options.setError(
          i18n.t("config.invalid_integer_fields", { fields: invalidKeys.join(", ") }),
        );
        return;
      }

      const context = options.getCompareContext();
      if (
        options.editorMode.value === "visual" &&
        context?.compareMode.value &&
        Object.keys(context.compareTargetNumericFieldErrors.value).length > 0
      ) {
        const invalidKeys = Object.keys(context.compareTargetNumericFieldErrors.value);
        options.setError(
          i18n.t("config.invalid_integer_fields", { fields: invalidKeys.join(", ") }),
        );
        return;
      }

      const pendingItems: PendingSaveItem[] = [];

      const sourceChanged =
        options.sourceDraftText.value !== options.loadedSourceText.value ||
        !areMapValuesEqual(options.editValues.value, options.loadedValues.value);
      if (sourceChanged) {
        const latestSourceText = await configApi.readServerConfigSource(
          options.serverPath.value,
          options.currentConfigRelativePath.value,
          options.currentConfigLocator.value || undefined,
          options.discoveryOptions.value,
        );
        const nextSourceText =
          options.editorMode.value === "visual" && options.currentConfigKind.value === "properties"
            ? await buildVisualPreviewSource()
            : options.sourceDraftText.value;

        if (nextSourceText !== latestSourceText) {
          pendingItems.push({
            serverId: options.currentServerId.value || "",
            serverName: options.currentServerName.value || i18n.t("config.compare.source_server"),
            serverPath: options.serverPath.value,
            filePath: options.currentConfigFilePath.value,
            relativePath: options.currentConfigRelativePath.value,
            locator: options.currentConfigLocator.value || undefined,
            discoveryOptions: options.discoveryOptions.value,
            originalText: latestSourceText,
            modifiedText: nextSourceText,
          });
        } else {
          await options.applyParsedSourceState(latestSourceText, options.editorMode.value);
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
        const latestTargetText = await configApi.readServerConfigSource(
          context.compareTargetPath.value,
          options.currentConfigRelativePath.value,
        );
        const nextTargetText =
          options.editorMode.value === "visual"
            ? await context.buildCompareTargetPreviewSource()
            : context.compareTargetSourceDraftText.value;

        if (nextTargetText !== latestTargetText) {
          pendingItems.push({
            serverId: context.compareTargetServerId.value,
            serverName:
              context.compareTargetServerName.value || i18n.t("config.compare.target_server"),
            serverPath: context.compareTargetPath.value,
            filePath: context.compareTargetServerPropertiesPath.value,
            relativePath: options.currentConfigRelativePath.value,
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

  async function confirmSaveProperties() {
    if (pendingSaveItems.value.length === 0 || options.saving.value) {
      return;
    }

    options.saving.value = true;
    options.setError(null);
    options.setSuccess(null);

    try {
      await Promise.all(
        pendingSaveItems.value.map((item) =>
          configApi.writeServerConfigSource(
            item.serverPath,
            item.relativePath,
            item.modifiedText,
            item.locator,
            item.discoveryOptions,
          ),
        ),
      );

      const savedCurrent = pendingSaveItems.value.find(
        (item) => item.serverId === options.currentServerId.value,
      );
      if (savedCurrent) {
        await options.applyParsedSourceState(savedCurrent.modifiedText, options.editorMode.value);
      }

      const context = options.getCompareContext();
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
      options.saving.value = false;
    }
  }

  function closeSaveDiffModal() {
    if (options.saving.value) {
      return;
    }

    pendingSaveItems.value = [];
    showSaveDiffModal.value = false;
  }

  return {
    showSaveDiffModal,
    pendingSaveItems,
    pendingSaveItemsWithStats,
    buildVisualPreviewSource,
    saveProperties,
    confirmSaveProperties,
    closeSaveDiffModal,
  };
}
