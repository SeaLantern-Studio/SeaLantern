import { ref, watch, type Ref } from "vue";
import { appendCustomCandidate } from "@components/views/create/createServerWorkflow";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import { serverApi } from "@api/server";
import { i18n } from "@language";

type SourceType = "archive" | "folder" | "";

interface UseCreateServerScanOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<SourceType>;
  showError: (message: string) => void;
}

export function useCreateServerScan(options: UseCreateServerScanOptions) {
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
  const startupSyncPending = ref(false);
  const startupCandidates = ref<StartupCandidate[]>([]);
  const selectedStartupId = ref("");
  const customStartupCommand = ref("");

  let startupDetectRequestId = 0;
  const AUTO_SCAN_DEBOUNCE_MS = 120;
  let startupDetectTimer: ReturnType<typeof setTimeout> | null = null;

  function resetScanState() {
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
  }

  async function refreshStartupCandidates(path: string, type: SourceType, forceReset: boolean) {
    const requestId = ++startupDetectRequestId;

    if (!path.trim() || !type) {
      resetScanState();
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
      options.showError(String(error));
    } finally {
      if (requestId === startupDetectRequestId) {
        coreDetecting.value = false;
        startupDetecting.value = false;
        startupSyncPending.value = false;
      }
    }
  }

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

    startupSyncPending.value = true;
    startupDetectTimer = setTimeout(() => {
      startupDetectTimer = null;
      void refreshStartupCandidates(path, type, false);
    }, AUTO_SCAN_DEBOUNCE_MS);
  }

  watch(
    [options.sourcePath, options.sourceType],
    ([path, type]) => {
      scheduleStartupDetect(path, type);
    },
    { immediate: true },
  );

  async function rescanStartupCandidates() {
    await refreshStartupCandidates(options.sourcePath.value.trim(), options.sourceType.value, true);
  }

  function cleanupScanTimer() {
    if (!startupDetectTimer) {
      return;
    }
    clearTimeout(startupDetectTimer);
    startupDetectTimer = null;
  }

  return {
    coreDetecting,
    detectedCoreType,
    detectedCoreMainClass,
    detectedCoreTypeKey,
    coreTypeOptions,
    selectedCoreType,
    detectedMcVersion,
    mcVersionOptions,
    selectedMcVersion,
    mcVersionDetectionFailed,
    startupDetecting,
    startupSyncPending,
    startupCandidates,
    selectedStartupId,
    customStartupCommand,
    refreshStartupCandidates,
    rescanStartupCandidates,
    cleanupScanTimer,
  };
}
