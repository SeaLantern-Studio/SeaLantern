import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { appendCustomCandidate } from "@components/views/create/createServerWorkflow";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import {
  containsIoRedirection,
  isStrictChildPath,
  mapStartupModeForModpack,
} from "@components/views/create/startupUtils";
import type { JavaInfo } from "@api/java";
import { javaApi } from "@api/java";
import { serverApi } from "@api/server";
import { settingsApi } from "@api/settings";
import { systemApi } from "@api/system";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";

type SourceType = "archive" | "folder" | "";

export function useCreateServerPage() {
  const router = useRouter();
  const store = useServerStore();
  const { error: errorMsg, showError, clearError } = useMessage();
  const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
  const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

  const sourcePath = ref("");
  const sourceType = ref<SourceType>("");
  const runPath = ref("");
  const useSoftwareDataDir = ref(false);

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

  const copyConflictDialogOpen = ref(false);
  const copyConflictItems = ref<string[]>([]);
  let copyConflictResolver: ((confirmed: boolean) => void) | null = null;

  const serverName = ref("My Server");
  const maxMemory = ref("2048");
  const minMemory = ref("512");
  const port = ref("25565");
  const selectedJava = ref("");
  const onlineMode = ref(true);
  const javaList = ref<JavaInfo[]>([]);

  const hasSource = computed(() => sourcePath.value.trim().length > 0 && sourceType.value !== "");
  const requiresRunPath = computed(() => sourceType.value === "archive");

  const selectedStartup = computed(
    () => startupCandidates.value.find((item) => item.id === selectedStartupId.value) ?? null,
  );
  const starterSelected = computed(() => selectedStartup.value?.mode === "starter");
  const customCommandHasRedirect = computed(
    () => selectedStartup.value?.mode === "custom" && containsIoRedirection(customStartupCommand.value),
  );

  const hasPathStep = computed(() => {
    if (!hasSource.value) {
      return false;
    }
    return requiresRunPath.value ? useSoftwareDataDir.value || runPath.value.trim().length > 0 : true;
  });

  const hasStartupStep = computed(() => {
    if (!hasPathStep.value || !selectedStartup.value) {
      return false;
    }
    if (selectedStartup.value.mode === "custom") {
      return customStartupCommand.value.trim().length > 0 && !customCommandHasRedirect.value;
    }
    if (
      selectedStartup.value.mode === "starter" &&
      mcVersionDetectionFailed.value &&
      selectedMcVersion.value.trim().length === 0
    ) {
      return false;
    }
    return true;
  });

  const hasJava = computed(() => selectedJava.value.trim().length > 0);
  const hasServerConfig = computed(() => serverName.value.trim().length > 0);

  const step1Completed = computed(() => hasSource.value);
  const step2Completed = computed(() => step1Completed.value && hasPathStep.value);
  const step3Completed = computed(() => step2Completed.value && hasStartupStep.value);
  const step4Completed = computed(() => step3Completed.value && hasJava.value && hasServerConfig.value);

  const activeStep = computed(() => {
    if (!step1Completed.value) {
      return 1;
    }
    if (!step2Completed.value) {
      return 2;
    }
    if (!step3Completed.value) {
      return 3;
    }
    if (!step4Completed.value) {
      return 4;
    }
    return 5;
  });

  const stepItems = computed(() => [
    {
      step: 1,
      title: i18n.t("create.step_source_title"),
      description: i18n.t("create.step_source_desc"),
      completed: step1Completed.value,
    },
    {
      step: 2,
      title: i18n.t("create.step_path_title"),
      description: i18n.t("create.step_path_desc"),
      completed: step2Completed.value,
    },
    {
      step: 3,
      title: i18n.t("create.step_startup_title"),
      description: i18n.t("create.step_startup_desc"),
      completed: step3Completed.value,
    },
    {
      step: 4,
      title: i18n.t("create.step_config_title"),
      description: i18n.t("create.step_config_desc"),
      completed: step4Completed.value,
    },
    {
      step: 5,
      title: i18n.t("create.step_action_title"),
      description: i18n.t("create.step_action_desc"),
      completed: false,
    },
  ]);

  // 只有步骤完成且“启动项同步”完成后才允许提交，避免新源路径配旧 startupFilePath。
  const canSubmit = computed(() => step4Completed.value && !startupSyncPending.value && !startupDetecting.value);

  onMounted(async () => {
    await loadDefaultSettings();
  });

  watch(sourceType, (nextType) => {
    if (nextType === "archive") {
      runPath.value = "";
    }
  });

  onUnmounted(() => {
    if (startupDetectTimer) {
      clearTimeout(startupDetectTimer);
      startupDetectTimer = null;
    }
  });

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

  function parseNumber(value: string, fallbackValue: number): number {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : fallbackValue;
  }

  async function loadDefaultSettings() {
    try {
      const settings = await settingsApi.get();

      maxMemory.value = String(settings.default_max_memory);
      minMemory.value = String(settings.default_min_memory);
      port.value = String(settings.default_port);

      if (settings.cached_java_list && settings.cached_java_list.length > 0) {
        javaList.value = settings.cached_java_list;
        if (settings.default_java_path) {
          selectedJava.value = settings.default_java_path;
        } else {
          const preferredJava = javaList.value.find((java) => java.is_64bit && java.major_version >= 17);
          selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
        }
      }
    } catch (error) {
      console.error("Failed to load default settings:", error);
    }
  }

  async function detectJava() {
    startJavaLoading();
    try {
      javaList.value = await javaApi.detect();
      if (javaList.value.length > 0) {
        const preferredJava = javaList.value.find((java) => java.is_64bit && java.major_version >= 17);
        selectedJava.value = preferredJava ? preferredJava.path : javaList.value[0].path;
      }

      const settings = await settingsApi.get();
      settings.cached_java_list = javaList.value;
      await settingsApi.save(settings);
    } catch (error) {
      showError(String(error));
    } finally {
      stopJavaLoading();
    }
  }

  async function pickRunPath() {
    const selected = await systemApi.pickFolder();
    if (selected) {
      updateRunPath(selected);
    }
  }

  function updateRunPath(nextPath: string) {
    const targetPath = nextPath.trim();
    if (
      sourceType.value === "folder" &&
      !useSoftwareDataDir.value &&
      isStrictChildPath(targetPath, sourcePath.value)
    ) {
      showError(i18n.t("create.path_child_of_source_forbidden"));
      return;
    }

    runPath.value = nextPath;
  }

  function toggleUseSoftwareDataDir() {
    useSoftwareDataDir.value = !useSoftwareDataDir.value;
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

      detectedCoreType.value = discovered.parsedCore.coreType || i18n.t("create.source_core_unknown");
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

  function validateBeforeSubmit(): boolean {
    clearError();

    if (!hasSource.value) {
      showError(i18n.t("create.source_required"));
      return false;
    }
    if (requiresRunPath.value && !useSoftwareDataDir.value && runPath.value.trim().length === 0) {
      showError(i18n.t("create.path_required_archive"));
      return false;
    }
    if (
      sourceType.value === "folder" &&
      !useSoftwareDataDir.value &&
      isStrictChildPath(runPath.value, sourcePath.value)
    ) {
      showError(i18n.t("create.path_child_of_source_forbidden"));
      return false;
    }
    if (!selectedStartup.value) {
      showError(i18n.t("create.startup_required"));
      return false;
    }

    if (selectedStartup.value.mode === "custom") {
      if (!customStartupCommand.value.trim()) {
        showError(i18n.t("create.startup_custom_required"));
        return false;
      }
      if (containsIoRedirection(customStartupCommand.value)) {
        showError(i18n.t("create.startup_custom_redirect_forbidden"));
        return false;
      }
    }

    if (
      selectedStartup.value.mode === "starter" &&
      mcVersionDetectionFailed.value &&
      selectedMcVersion.value.trim().length === 0
    ) {
      showError(i18n.t("create.startup_mc_version_required"));
      return false;
    }

    if (!selectedJava.value) {
      showError(i18n.t("common.select_java_path"));
      return false;
    }
    if (!serverName.value.trim()) {
      showError(i18n.t("common.enter_server_name"));
      return false;
    }

    return true;
  }

  function confirmCopyConflict() {
    copyConflictDialogOpen.value = false;
    copyConflictResolver?.(true);
    copyConflictResolver = null;
  }

  function cancelCopyConflict() {
    copyConflictDialogOpen.value = false;
    copyConflictResolver?.(false);
    copyConflictResolver = null;
  }

  async function handleSubmit() {
    if (!validateBeforeSubmit()) {
      return;
    }

    startCreating();
    try {
      const startup = selectedStartup.value;
      const startupMode = mapStartupModeForModpack(startup?.mode ?? "jar");
      const resolvedCoreType = selectedCoreType.value.trim() || detectedCoreTypeKey.value.trim();
      const resolvedMcVersion =
        startupMode === "starter"
          ? selectedMcVersion.value.trim() || detectedMcVersion.value.trim()
          : "";
      await serverApi.importModpack({
        name: serverName.value.trim(),
        modpackPath: sourcePath.value,
        javaPath: selectedJava.value,
        maxMemory: parseNumber(maxMemory.value, 2048),
        minMemory: parseNumber(minMemory.value, 512),
        port: parseNumber(port.value, 25565),
        startupMode,
        onlineMode: onlineMode.value,
        customCommand: startupMode === "custom" ? customStartupCommand.value.trim() : undefined,
        runPath: runPath.value.trim(),
        useSoftwareDataDir: useSoftwareDataDir.value,
        startupFilePath: startupMode === "custom" ? undefined : startup?.path,
        coreType: resolvedCoreType || undefined,
        mcVersion: resolvedMcVersion || undefined,
      });

      await store.refreshList();
      router.push("/");
    } catch (error) {
      showError(String(error));
    } finally {
      stopCreating();
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
    useSoftwareDataDir,
    coreDetecting,
    detectedCoreType,
    detectedCoreMainClass,
    startupDetecting,
    startupCandidates,
    selectedStartupId,
    customStartupCommand,
    starterSelected,
    detectedCoreTypeKey,
    coreTypeOptions,
    selectedCoreType,
    detectedMcVersion,
    mcVersionOptions,
    selectedMcVersion,
    mcVersionDetectionFailed,
    customCommandHasRedirect,
    copyConflictDialogOpen,
    copyConflictItems,
    serverName,
    maxMemory,
    minMemory,
    port,
    selectedJava,
    onlineMode,
    javaList,
    activeStep,
    stepItems,
    canSubmit,
    pickRunPath,
    updateRunPath,
    toggleUseSoftwareDataDir,
    rescanStartupCandidates,
    detectJava,
    handleSubmit,
    confirmCopyConflict,
    cancelCopyConflict,
  };
}
