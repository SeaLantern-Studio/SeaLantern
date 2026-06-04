import { useCreateServerDefaults } from "@components/views/create/useCreateServerDefaults";
import {
  type CreateServerSourceType,
  useCreateServerDropSource,
} from "@components/views/create/useCreateServerDropSource";
import { useCreateServerScan } from "@components/views/create/useCreateServerScan";
import { useCreateServerSubmit } from "@components/views/create/useCreateServerSubmit";
import { isStrictChildPath, normalizePathForCompare } from "@components/views/create/startupUtils";
import type { JavaInfo } from "@api/java";
import { javaApi } from "@api/java";
import { serverApi } from "@api/server";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { i18n } from "@language";
import { useCreateServerDraftStore } from "@stores/createServerDraft.ts";
import { onActivated, onUnmounted, ref, watch } from "vue";

export function useCreateServerPage() {
  const { error: errorMsg, showError, clearError } = useMessage();
  const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
  const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

  const sourcePath = ref("");
  const sourceType = ref<CreateServerSourceType>("");
  const runPath = ref("");

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

  const scan = useCreateServerScan({
    sourcePath,
    sourceType,
    showError,
  });

  useCreateServerDropSource({
    sourcePath,
    sourceType,
  });

  const submit = useCreateServerSubmit({
    sourcePath,
    sourceType,
    runPath,
    startupSyncPending: scan.startupSyncPending,
    startupDetecting: scan.startupDetecting,
    startupCandidates: scan.startupCandidates,
    selectedStartupId: scan.selectedStartupId,
    customStartupCommand: scan.customStartupCommand,
    detectedCoreTypeKey: scan.detectedCoreTypeKey,
    selectedCoreType: scan.selectedCoreType,
    detectedMcVersion: scan.detectedMcVersion,
    selectedMcVersion: scan.selectedMcVersion,
    mcVersionDetectionFailed: scan.mcVersionDetectionFailed,
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

  onUnmounted(() => {
    scan.cleanupScanTimer();
    if (runPathConflictTimer) {
      clearTimeout(runPathConflictTimer);
      runPathConflictTimer = null;
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

  async function rescanStartupCandidates() {
    await scan.rescanStartupCandidates();
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
    coreDetecting: scan.coreDetecting,
    detectedCoreType: scan.detectedCoreType,
    detectedCoreMainClass: scan.detectedCoreMainClass,
    startupDetecting: scan.startupDetecting,
    startupCandidates: scan.startupCandidates,
    selectedStartupId: scan.selectedStartupId,
    customStartupCommand: scan.customStartupCommand,
    detectedCoreTypeKey: scan.detectedCoreTypeKey,
    coreTypeOptions: scan.coreTypeOptions,
    selectedCoreType: scan.selectedCoreType,
    detectedMcVersion: scan.detectedMcVersion,
    mcVersionOptions: scan.mcVersionOptions,
    selectedMcVersion: scan.selectedMcVersion,
    mcVersionDetectionFailed: scan.mcVersionDetectionFailed,
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
    starterSelected: submit.starterSelected,
    customCommandHasRedirect: submit.customCommandHasRedirect,
  };
}
