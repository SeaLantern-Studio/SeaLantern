import { computed, onBeforeUnmount, shallowRef, watch } from "vue";
import { configApi, type DiscoveredServerConfigFile } from "@api/config";
import { i18n } from "@language";
import { extractMotdFromSource, updateMotdInSource } from "@src/features/config-editor/motdFormat";
import { useNextInstanceWorkspaceContext } from "@src/composables/useNextInstanceWorkspace";

type PendingDiscardAction = { type: "switch"; locator: string } | { type: "reload-current" } | null;

export type ServerInstanceConfigPreviewState =
  | { kind: "empty"; message: string }
  | { kind: "unsupported"; message: string }
  | { kind: "loading"; message: string; note: string }
  | { kind: "error"; message: string; note?: string }
  | { kind: "ready"; modified: string; note: string };

function isPrimaryServerProperties(file: DiscoveredServerConfigFile | null): boolean {
  return (
    !!file &&
    file.known_role === "server_properties" &&
    file.kind === "properties" &&
    file.source_kind === "server_root"
  );
}

function supportsPreview(file: DiscoveredServerConfigFile | null): boolean {
  return isPrimaryServerProperties(file);
}

function sortDiscoveredFiles(files: DiscoveredServerConfigFile[]): DiscoveredServerConfigFile[] {
  return files.toSorted((left, right) => {
    return (
      left.priority - right.priority ||
      left.relative_path.localeCompare(right.relative_path) ||
      left.locator.localeCompare(right.locator)
    );
  });
}

function pickPreferredFile(
  files: DiscoveredServerConfigFile[],
  preferredLocator: string,
): DiscoveredServerConfigFile | null {
  if (preferredLocator) {
    const matched = files.find((file) => file.locator === preferredLocator) ?? null;
    if (matched) {
      return matched;
    }
  }

  return (
    files.find((file) => file.known_role === "server_properties") ??
    files.find((file) => file.known_role === "startup_primary") ??
    files[0] ??
    null
  );
}

export function useServerInstanceConfigPage() {
  const workspace = useNextInstanceWorkspaceContext();

  const files = shallowRef<DiscoveredServerConfigFile[]>([]);
  const loadingFiles = shallowRef(false);
  const refreshingFiles = shallowRef(false);
  const loadingCurrentFile = shallowRef(false);
  const saving = shallowRef(false);
  const selectedLocator = shallowRef("");
  const lastLoadedLocator = shallowRef("");
  const loadedSource = shallowRef("");
  const draftSource = shallowRef("");
  const errorMessage = shallowRef("");
  const successMessage = shallowRef("");
  const pendingDiscardAction = shallowRef<PendingDiscardAction>(null);
  const motdEditorVisible = shallowRef(false);
  const motdDraft = shallowRef("");
  const previewState = shallowRef<ServerInstanceConfigPreviewState>({
    kind: "empty",
    message: i18n.t("config.next_v1.preview_empty"),
  });

  let previewRequestId = 0;
  let previewTimer: number | null = null;
  let initializeRequestId = 0;
  let activeSuccessTimer: number | null = null;

  const serverId = computed(() => workspace.serverId.value);
  const server = computed(() => workspace.server.value);
  const serverPath = computed(() => workspace.server.value?.path ?? "");
  const currentFile = computed(
    () => files.value.find((file) => file.locator === selectedLocator.value) ?? null,
  );
  const canEditMotd = computed(() => isPrimaryServerProperties(currentFile.value));
  const hasUnsavedChanges = computed(() => draftSource.value !== loadedSource.value);
  const hasFiles = computed(() => files.value.length > 0);
  const previewSource = computed(() => draftSource.value);

  const discardDialogState = computed(() => {
    const pending = pendingDiscardAction.value;
    if (!pending) {
      return {
        visible: false,
        title: "",
        message: "",
        confirmText: "",
      };
    }

    const isSwitch = pending.type === "switch";
    return {
      visible: true,
      title: i18n.t("config.next_v1.unsaved_title"),
      message: i18n.t(
        isSwitch
          ? "config.next_v1.unsaved_switch_message"
          : "config.next_v1.unsaved_reload_message",
      ),
      confirmText: i18n.t(
        isSwitch
          ? "config.next_v1.unsaved_confirm_switch"
          : "config.next_v1.unsaved_confirm_reload",
      ),
    };
  });

  function clearSuccessMessageLater() {
    if (activeSuccessTimer !== null) {
      window.clearTimeout(activeSuccessTimer);
    }

    activeSuccessTimer = window.setTimeout(() => {
      successMessage.value = "";
      activeSuccessTimer = null;
    }, 2400);
  }

  function clearPreviewTimer() {
    if (previewTimer !== null) {
      window.clearTimeout(previewTimer);
      previewTimer = null;
    }
  }

  async function readSourceForFile(file: DiscoveredServerConfigFile): Promise<string> {
    if (isPrimaryServerProperties(file)) {
      return configApi.readServerPropertiesSource(serverPath.value);
    }

    return configApi.readServerConfigSource(serverPath.value, file.relative_path, file.locator);
  }

  async function writeSourceForFile(
    file: DiscoveredServerConfigFile,
    source: string,
  ): Promise<void> {
    if (isPrimaryServerProperties(file)) {
      await configApi.writeServerPropertiesSource(serverPath.value, source);
      return;
    }

    await configApi.writeServerConfigSource(
      serverPath.value,
      file.relative_path,
      source,
      file.locator,
    );
  }

  async function loadCurrentFile(file: DiscoveredServerConfigFile): Promise<void> {
    loadingCurrentFile.value = true;
    errorMessage.value = "";

    try {
      const source = await readSourceForFile(file);
      loadedSource.value = source;
      draftSource.value = source;
      lastLoadedLocator.value = file.locator;
    } catch (error) {
      loadedSource.value = "";
      draftSource.value = "";
      lastLoadedLocator.value = "";
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingCurrentFile.value = false;
    }
  }

  async function loadFileList(options?: {
    preserveSelection?: boolean;
    forceReadCurrent?: boolean;
    showRefreshing?: boolean;
  }): Promise<void> {
    const preserveSelection = options?.preserveSelection ?? true;
    const forceReadCurrent = options?.forceReadCurrent ?? false;
    const showRefreshing = options?.showRefreshing ?? false;

    if (!serverPath.value) {
      files.value = [];
      selectedLocator.value = "";
      lastLoadedLocator.value = "";
      loadedSource.value = "";
      draftSource.value = "";
      previewState.value = {
        kind: "empty",
        message: i18n.t("config.next_v1.preview_empty"),
      };
      return;
    }

    if (showRefreshing) {
      refreshingFiles.value = true;
    } else {
      loadingFiles.value = true;
    }

    try {
      const discovered = sortDiscoveredFiles(
        await configApi.listServerConfigFiles(serverPath.value),
      );
      files.value = discovered;

      const nextFile = pickPreferredFile(
        discovered,
        preserveSelection ? selectedLocator.value : "",
      );

      if (!nextFile) {
        selectedLocator.value = "";
        lastLoadedLocator.value = "";
        loadedSource.value = "";
        draftSource.value = "";
        previewState.value = {
          kind: "empty",
          message: i18n.t("config.next_v1.preview_empty"),
        };
        return;
      }

      const selectionChanged = selectedLocator.value !== nextFile.locator;
      selectedLocator.value = nextFile.locator;

      if (selectionChanged || forceReadCurrent || lastLoadedLocator.value !== nextFile.locator) {
        await loadCurrentFile(nextFile);
      }
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingFiles.value = false;
      refreshingFiles.value = false;
    }
  }

  async function initializePage(): Promise<void> {
    const requestId = ++initializeRequestId;
    errorMessage.value = "";
    successMessage.value = "";

    await workspace.refreshServerContext();
    if (requestId !== initializeRequestId) {
      return;
    }

    await loadFileList({
      preserveSelection: false,
      forceReadCurrent: true,
    });
  }

  async function performSwitchFile(locator: string): Promise<void> {
    if (!locator || locator === selectedLocator.value) {
      return;
    }

    const file = files.value.find((item) => item.locator === locator) ?? null;
    if (!file) {
      return;
    }

    selectedLocator.value = locator;
    await loadCurrentFile(file);
  }

  async function requestSwitchFile(locator: string): Promise<void> {
    if (!locator || locator === selectedLocator.value) {
      return;
    }

    if (hasUnsavedChanges.value) {
      pendingDiscardAction.value = { type: "switch", locator };
      return;
    }

    await performSwitchFile(locator);
  }

  async function reloadCurrentFile(): Promise<void> {
    const file = currentFile.value;
    if (!file) {
      return;
    }

    if (hasUnsavedChanges.value) {
      pendingDiscardAction.value = { type: "reload-current" };
      return;
    }

    await loadCurrentFile(file);
  }

  async function confirmDiscardAndContinue(): Promise<void> {
    const pending = pendingDiscardAction.value;
    pendingDiscardAction.value = null;
    if (!pending) {
      return;
    }

    if (pending.type === "switch") {
      await performSwitchFile(pending.locator);
      return;
    }

    const file = currentFile.value;
    if (file) {
      await loadCurrentFile(file);
    }
  }

  function cancelDiscardAndContinue(): void {
    pendingDiscardAction.value = null;
  }

  function syncMotdDraftFromSource(source: string): void {
    motdDraft.value = extractMotdFromSource(source);
  }

  function updateDraftSource(value: string): void {
    draftSource.value = value;
    if (motdEditorVisible.value && canEditMotd.value) {
      syncMotdDraftFromSource(value);
    }
  }

  function openMotdEditor(): void {
    if (!canEditMotd.value) {
      return;
    }

    syncMotdDraftFromSource(draftSource.value);
    motdEditorVisible.value = true;
  }

  function closeMotdEditor(): void {
    motdEditorVisible.value = false;
  }

  function updateMotdDraft(value: string): void {
    motdDraft.value = value;
  }

  async function saveMotdDraft(value: string): Promise<void> {
    draftSource.value = updateMotdInSource(draftSource.value, value);
    syncMotdDraftFromSource(draftSource.value);
    await saveCurrentFile();
    if (!errorMessage.value) {
      motdEditorVisible.value = false;
    }
  }

  async function refreshFiles(): Promise<void> {
    await loadFileList({
      preserveSelection: true,
      forceReadCurrent: false,
      showRefreshing: true,
    });
  }

  async function saveCurrentFile(): Promise<void> {
    const file = currentFile.value;
    if (!file || !hasUnsavedChanges.value) {
      return;
    }

    saving.value = true;
    errorMessage.value = "";
    successMessage.value = "";

    try {
      await writeSourceForFile(file, draftSource.value);
      await loadCurrentFile(file);
      successMessage.value = i18n.t("config.next_v1.saved_source");
      clearSuccessMessageLater();
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      saving.value = false;
    }
  }

  async function refreshPreviewNow(): Promise<void> {
    const file = currentFile.value;
    if (!file) {
      previewState.value = {
        kind: "empty",
        message: i18n.t("config.next_v1.preview_empty"),
      };
      return;
    }

    if (!supportsPreview(file)) {
      previewState.value = {
        kind: "unsupported",
        message: i18n.t("config.next_v1.preview_unsupported"),
      };
      return;
    }

    const requestId = ++previewRequestId;
    const previewNote = i18n.t("config.next_v1.preview_note");

    previewState.value = {
      kind: "loading",
      message: i18n.t("config.next_v1.preview_loading"),
      note: previewNote,
    };

    try {
      const parsed = await configApi.parseServerPropertiesSource(draftSource.value);
      const modified = await configApi.previewServerPropertiesWriteFromSource(
        draftSource.value,
        parsed.raw,
      );

      if (requestId !== previewRequestId) {
        return;
      }

      previewState.value = {
        kind: "ready",
        modified,
        note: previewNote,
      };
    } catch (error) {
      if (requestId !== previewRequestId) {
        return;
      }

      previewState.value = {
        kind: "error",
        message:
          error instanceof Error ? error.message : i18n.t("config.next_v1.preview_parse_failed"),
        note: previewNote,
      };
    }
  }

  function schedulePreviewRefresh(): void {
    clearPreviewTimer();

    if (!currentFile.value) {
      previewState.value = {
        kind: "empty",
        message: i18n.t("config.next_v1.preview_empty"),
      };
      return;
    }

    if (!supportsPreview(currentFile.value)) {
      previewState.value = {
        kind: "unsupported",
        message: i18n.t("config.next_v1.preview_unsupported"),
      };
      return;
    }

    previewTimer = window.setTimeout(() => {
      void refreshPreviewNow();
    }, 220);
  }

  watch(
    serverId,
    () => {
      void initializePage();
    },
    { immediate: true },
  );

  watch([currentFile, draftSource], () => {
    schedulePreviewRefresh();
  });

  watch(currentFile, (file) => {
    if (!isPrimaryServerProperties(file)) {
      motdEditorVisible.value = false;
      motdDraft.value = "";
    }
  });

  onBeforeUnmount(() => {
    clearPreviewTimer();
    if (activeSuccessTimer !== null) {
      window.clearTimeout(activeSuccessTimer);
    }
  });

  return {
    server,
    files,
    hasFiles,
    loadingFiles,
    refreshingFiles,
    loadingCurrentFile,
    saving,
    currentFile,
    selectedLocator,
    loadedSource,
    draftSource,
    previewSource,
    previewState,
    motdEditorVisible,
    motdDraft,
    canEditMotd,
    hasUnsavedChanges,
    errorMessage,
    successMessage,
    discardDialogState,
    updateDraftSource,
    openMotdEditor,
    closeMotdEditor,
    updateMotdDraft,
    saveMotdDraft,
    requestSwitchFile,
    refreshFiles,
    reloadCurrentFile,
    saveCurrentFile,
    confirmDiscardAndContinue,
    cancelDiscardAndContinue,
  };
}
