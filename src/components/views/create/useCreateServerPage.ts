import { onActivated, onMounted, onUnmounted, ref, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCreateServerDefaults } from "@components/views/create/useCreateServerDefaults";
import { useCreateServerSubmit } from "@components/views/create/useCreateServerSubmit";
import { appendCustomCandidate } from "@components/views/create/createServerWorkflow";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import { isStrictChildPath, normalizePathForCompare } from "@components/views/create/startupUtils";
import type { JavaInfo } from "@api/java";
import { javaApi } from "@api/java";
import { serverApi } from "@api/server";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { i18n } from "@language";
import { useCreateServerDraftStore } from "@stores/createServerDraft.ts";
import { isBrowserEnv } from "@api/tauri";

type SourceType = "archive" | "folder" | "";

function inferSourceType(path: string): SourceType {
  const lowerPath = path.toLowerCase();
  if (
    lowerPath.endsWith(".zip") ||
    lowerPath.endsWith(".tar") ||
    lowerPath.endsWith(".tar.gz") ||
    lowerPath.endsWith(".tgz") ||
    lowerPath.endsWith(".jar")
  ) {
    return "archive";
  }
  return "folder";
}

export const CREATE_SERVER_SOURCE_DROP_EVENT = "create-server-source-drop";
const CREATE_SERVER_DND_DEBUG = import.meta.env.DEV;

function logCreateServerDnd(message: string, payload?: unknown) {
  if (!CREATE_SERVER_DND_DEBUG) return;
  if (payload === undefined) {
    console.debug(message);
    return;
  }
  console.debug(message, payload);
}

export function useCreateServerPage() {
  const { error: errorMsg, showError, clearError } = useMessage();
  const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
  const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

  const sourcePath = ref("");
  const sourceType = ref<SourceType>("");
  const runPath = ref("");

  const coreDetecting = ref(false);
  const detectedCoreType = ref("");
  const detectedCoreMainClass = ref("");
  const detectedCoreTypeKey = ref("");
  const coreTypeOptions = ref<string[]>([]);
  const selectedCoreType = ref("");

  const detectedMcVersion = ref("");
  const mcVersionOptions = ref<string[]>([]);
  const selectedMcVersion = ref("");
  const mcVersionDetectionFailed = ref(false);

  const startupDetecting = ref(false);
  // 源路径变更后先进入同步等待态，覆盖防抖空窗，防止提交时仍引用旧启动项。
  const startupSyncPending = ref(false);
  const startupCandidates = ref<StartupCandidate[]>([]);
  const selectedStartupId = ref("");
  const customStartupCommand = ref("");
  let startupDetectRequestId = 0;

  const AUTO_SCAN_DEBOUNCE_MS = 120;
  let startupDetectTimer: ReturnType<typeof setTimeout> | null = null;
  let unlistenSourceDropEvent: UnlistenFn | null = null;

  const runPathOverwriteRisk = ref(false);
  const RUN_PATH_CONFLICT_DEBOUNCE_MS = 180;
  let runPathConflictTimer: ReturnType<typeof setTimeout> | null = null;
  let runPathConflictRequestId = 0;

  const serverName = ref("My Server");
  const maxMemory = ref("2048");
  const minMemory = ref("512");
  const port = ref("25565");
  const selectedJava = ref("");
  const onlineMode = ref(true);
  const javaList = ref<JavaInfo[]>([]);

  const defaults = useCreateServerDefaults({
    sourcePath,
    sourceType,
    runPath,
    maxMemory,
    minMemory,
    port,
    selectedJava,
    javaList,
    isChildPathBlocked: (targetPath) => isStrictChildPath(targetPath, sourcePath.value),
    onInvalidRunPath: () => {
      showError(i18n.t("create.path_child_of_source_forbidden"));
    },
  });

  const submit = useCreateServerSubmit({
    sourcePath,
    sourceType,
    runPath,
    startupSyncPending,
    startupDetecting,
    startupCandidates,
    selectedStartupId,
    customStartupCommand,
    detectedCoreTypeKey,
    selectedCoreType,
    detectedMcVersion,
    selectedMcVersion,
    mcVersionDetectionFailed,
    serverName,
    maxMemory,
    minMemory,
    port,
    selectedJava,
    onlineMode,
    startCreating,
    stopCreating,
    showError,
    clearError,
  });

  onActivated(() => {
    loadFromDraft();
  });

  onMounted(async () => {
    if (!isBrowserEnv()) {
      try {
        unlistenSourceDropEvent = await listen<string[]>(
          CREATE_SERVER_SOURCE_DROP_EVENT,
          (event) => {
            const droppedPaths = Array.isArray(event.payload) ? event.payload : [];
            logCreateServerDnd("[useCreateServerPage] Received source drop event", droppedPaths);
            if (droppedPaths.length === 0) {
              return;
            }

            const path = droppedPaths[0];
            sourcePath.value = path;
            sourceType.value = inferSourceType(path);
          },
        );
      } catch (error) {
        logCreateServerDnd("[useCreateServerPage] Failed to register source drop listener", error);
      }
    }
  });

  onUnmounted(() => {
    if (startupDetectTimer) {
      clearTimeout(startupDetectTimer);
      startupDetectTimer = null;
    }
    if (runPathConflictTimer) {
      clearTimeout(runPathConflictTimer);
      runPathConflictTimer = null;
    }
    if (unlistenSourceDropEvent) {
      unlistenSourceDropEvent();
      unlistenSourceDropEvent = null;
    }
  });

  function scheduleRunPathConflictCheck() {
    if (runPathConflictTimer) {
      clearTimeout(runPathConflictTimer);
      runPathConflictTimer = null;
    }

    const sourceDir = sourcePath.value.trim();
    const targetDir = runPath.value.trim();
    if (!sourceDir || !targetDir || sourceType.value !== "folder") {
      runPathConflictRequestId += 1;
      runPathOverwriteRisk.value = false;
      return;
    }

    if (normalizePathForCompare(sourceDir) === normalizePathForCompare(targetDir)) {
      runPathConflictRequestId += 1;
      runPathOverwriteRisk.value = false;
      return;
    }

    const requestId = ++runPathConflictRequestId;
    runPathConflictTimer = setTimeout(() => {
      runPathConflictTimer = null;
      void checkRunPathConflict(sourceDir, targetDir, requestId);
    }, RUN_PATH_CONFLICT_DEBOUNCE_MS);
  }

  async function checkRunPathConflict(sourceDir: string, targetDir: string, requestId: number) {
    try {
      const conflicts = await serverApi.collectCopyConflicts(sourceDir, targetDir);
      if (requestId !== runPathConflictRequestId) {
        return;
      }
      runPathOverwriteRisk.value = conflicts.length > 0;
    } catch (error) {
      if (requestId !== runPathConflictRequestId) {
        return;
      }
      runPathOverwriteRisk.value = false;
      console.error("Failed to check run path conflict:", error);
    }
  }

  watch(
    [sourceType, sourcePath, runPath],
    () => {
      scheduleRunPathConflictCheck();
    },
    { immediate: true },
  );

  function scheduleStartupDetect(path: string, type: SourceType) {
    if (startupDetectTimer) {
      clearTimeout(startupDetectTimer);
      startupDetectTimer = null;
    }

    if (!path.trim() || !type) {
      startupSyncPending.value = false;
      void refreshStartupCandidates(path, type, false);
      return;
    }

    // 源路径一旦变化就立刻锁提交，直到本轮扫描完成。
    startupSyncPending.value = true;

    startupDetectTimer = setTimeout(() => {
      startupDetectTimer = null;
      void refreshStartupCandidates(path, type, false);
    }, AUTO_SCAN_DEBOUNCE_MS);
  }

  watch(
    [sourcePath, sourceType],
    ([path, type]) => {
      scheduleStartupDetect(path, type);
    },
    { immediate: true },
  );

  function loadFromDraft() {
    const draftStore = useCreateServerDraftStore();
    const draft = draftStore.consumeDraft();
    if (draft !== null) {
      sourcePath.value = draft.sourcePath;
      sourceType.value = draft.sourceType;
    }
  }

  async function detectJava() {
    startJavaLoading();
    try {
      javaList.value = await javaApi.detect();
      if (javaList.value.length > 0) {
        const preferredJava = javaList.value.find(
          (java) => java.is_64bit && java.major_version >= 17,
        );
        selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
      }
    } catch (error) {
      showError(String(error));
    } finally {
      stopJavaLoading();
    }
  }

  function updateRunPath(nextPath: string) {
    defaults.applyRunPath(nextPath);
  }

  async function refreshStartupCandidates(path: string, type: SourceType, forceReset: boolean) {
    const requestId = ++startupDetectRequestId;

    if (!path.trim() || !type) {
      coreDetecting.value = false;
      detectedCoreType.value = "";
      detectedCoreMainClass.value = "";
      startupDetecting.value = false;
      startupCandidates.value = [];
      selectedStartupId.value = "";
      customStartupCommand.value = "";
      detectedCoreTypeKey.value = "";
      coreTypeOptions.value = [];
      selectedCoreType.value = "";
      detectedMcVersion.value = "";
      mcVersionOptions.value = [];
      selectedMcVersion.value = "";
      mcVersionDetectionFailed.value = false;
      startupSyncPending.value = false;
      return;
    }

    coreDetecting.value = true;
    startupDetecting.value = true;
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    if (requestId !== startupDetectRequestId) {
      return;
    }
    try {
      const discovered = await serverApi.scanStartupCandidates(path, type as "archive" | "folder");
      const list = appendCustomCandidate(discovered.candidates);

      if (requestId !== startupDetectRequestId) {
        return;
      }

      detectedCoreType.value =
        discovered.parsedCore.coreType || i18n.t("create.source_core_unknown");
      detectedCoreMainClass.value = discovered.parsedCore.mainClass ?? "";
      const previousDetectedCoreKey = detectedCoreTypeKey.value;
      const previousDetectedMcVersion = detectedMcVersion.value;
      detectedCoreTypeKey.value = discovered.detectedCoreTypeKey ?? "";
      coreTypeOptions.value = discovered.coreTypeOptions;
      detectedMcVersion.value = discovered.detectedMcVersion ?? "";
      mcVersionOptions.value = discovered.mcVersionOptions;
      mcVersionDetectionFailed.value = discovered.mcVersionDetectionFailed;
      startupCandidates.value = list;

      if (forceReset || !list.some((item) => item.id === selectedStartupId.value)) {
        selectedStartupId.value = list[0]?.id ?? "";
      }

      if (
        forceReset ||
        !coreTypeOptions.value.includes(selectedCoreType.value) ||
        selectedCoreType.value === previousDetectedCoreKey
      ) {
        selectedCoreType.value = detectedCoreTypeKey.value;
      }

      if (
        forceReset ||
        !mcVersionOptions.value.includes(selectedMcVersion.value) ||
        selectedMcVersion.value === previousDetectedMcVersion
      ) {
        selectedMcVersion.value = detectedMcVersion.value;
      }
    } catch (error) {
      if (requestId !== startupDetectRequestId) {
        return;
      }
      detectedCoreType.value = i18n.t("create.source_core_unknown");
      detectedCoreMainClass.value = "";
      startupCandidates.value = appendCustomCandidate([]);
      selectedStartupId.value = startupCandidates.value[0]?.id ?? "";
      detectedCoreTypeKey.value = "";
      coreTypeOptions.value = [];
      selectedCoreType.value = "";
      detectedMcVersion.value = "";
      mcVersionOptions.value = [];
      selectedMcVersion.value = "";
      mcVersionDetectionFailed.value = false;
      showError(String(error));
    } finally {
      if (requestId === startupDetectRequestId) {
        coreDetecting.value = false;
        startupDetecting.value = false;
        startupSyncPending.value = false;
      }
    }
  }

  async function rescanStartupCandidates() {
    await refreshStartupCandidates(sourcePath.value.trim(), sourceType.value, true);
  }

  /**
   * 处理 Tauri 文件拖放事件
   * 根据文件扩展名自动识别为压缩包或文件夹
   */
  function handleTauriDrop(paths: string[]) {
    if (paths.length === 0) return;

    const archiveExtensions = [".zip", ".tar", ".tar.gz", ".tgz", ".jar"];

    function hasArchiveExtension(path: string): boolean {
      const lowerPath = path.toLowerCase();
      return archiveExtensions.some((ext) => lowerPath.endsWith(ext));
    }

    const firstPath = paths[0];
    if (hasArchiveExtension(firstPath)) {
      sourcePath.value = firstPath;
      sourceType.value = "archive";
    } else {
      sourcePath.value = firstPath;
      sourceType.value = "folder";
    }
  }

  return {
    errorMsg,
    clearError,
    showError,
    javaLoading,
    creating,
    sourcePath,
    sourceType,
    runPath,
    runPathOverwriteRisk,
    coreDetecting,
    detectedCoreType,
    detectedCoreMainClass,
    startupDetecting,
    startupCandidates,
    selectedStartupId,
    customStartupCommand,
    detectedCoreTypeKey,
    coreTypeOptions,
    selectedCoreType,
    detectedMcVersion,
    mcVersionOptions,
    selectedMcVersion,
    mcVersionDetectionFailed,
    serverName,
    maxMemory,
    minMemory,
    port,
    selectedJava,
    onlineMode,
    javaList,
    activeStep: submit.activeStep,
    stepItems: submit.stepItems,
    canSubmit: submit.canSubmit,
    pickRunPath: defaults.pickRunPath,
    updateRunPath,
    rescanStartupCandidates,
    detectJava,
    handleSubmit: submit.handleSubmit,
    handleTauriDrop,
    starterSelected: submit.starterSelected,
    customCommandHasRedirect: submit.customCommandHasRedirect,
  };
}
