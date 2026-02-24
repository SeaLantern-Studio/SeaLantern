import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import {
  appendCustomCandidate,
  buildArchiveStartupCandidates,
  buildFolderStartupCandidates,
  collectCopyConflicts,
  copyDirectoryRecursive,
} from "@components/views/create/createServerWorkflow";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import {
  containsIoRedirection,
  mapStartupModeForApi,
  normalizePathForCompare,
  resolveExecutablePathForTarget,
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
  const isWindows = navigator.userAgent.toLowerCase().includes("windows");

  const router = useRouter();
  const store = useServerStore();
  const { error: errorMsg, showError, clearError } = useMessage();
  const { loading: javaLoading, start: startJavaLoading, stop: stopJavaLoading } = useLoading();
  const { loading: creating, start: startCreating, stop: stopCreating } = useLoading();

  const sourcePath = ref("");
  const sourceType = ref<SourceType>("");
  const runPath = ref("");

  const coreDetecting = ref(false);
  const detectedCoreType = ref("");
  const detectedCoreMainClass = ref("");
  let coreDetectRequestId = 0;

  const startupDetecting = ref(false);
  const startupCandidates = ref<StartupCandidate[]>([]);
  const selectedStartupId = ref("");
  const customStartupCommand = ref("");
  let startupDetectRequestId = 0;

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
  const customCommandHasRedirect = computed(
    () => selectedStartup.value?.mode === "custom" && containsIoRedirection(customStartupCommand.value),
  );

  const resolvedFolderServerPath = computed(() => {
    if (sourceType.value !== "folder") {
      return "";
    }
    const source = sourcePath.value.trim();
    const target = runPath.value.trim();
    if (!target || normalizePathForCompare(source) === normalizePathForCompare(target)) {
      return source;
    }
    return target;
  });

  const hasPathStep = computed(() => {
    if (!hasSource.value) {
      return false;
    }
    return requiresRunPath.value ? runPath.value.trim().length > 0 : true;
  });

  const hasStartupStep = computed(() => {
    if (!hasPathStep.value || !selectedStartup.value) {
      return false;
    }
    if (selectedStartup.value.mode !== "custom") {
      return true;
    }
    return customStartupCommand.value.trim().length > 0 && !customCommandHasRedirect.value;
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
    return 4;
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
  ]);

  const canSubmit = computed(() => step4Completed.value && selectedStartup.value?.mode !== "custom");

  const showSourceCoreInfo = computed(() => sourcePath.value.trim().length > 0 && sourceType.value !== "");
  const sourceCoreInfoText = computed(() => {
    const coreName = detectedCoreType.value || i18n.t("create.source_core_unknown");
    const mainClass = detectedCoreMainClass.value;
    const suffix = mainClass ? `（${mainClass}）` : "（）";
    return `${i18n.t("create.source_core_label")}${coreName}${suffix}`;
  });

  onMounted(async () => {
    await loadDefaultSettings();
  });

  watch(sourceType, (nextType) => {
    if (nextType === "archive") {
      runPath.value = "";
    }
  });

  watch(
    [sourcePath, sourceType],
    async ([path, type]) => {
      const requestId = ++coreDetectRequestId;

      if (!path.trim() || !type) {
        coreDetecting.value = false;
        detectedCoreType.value = "";
        detectedCoreMainClass.value = "";
        return;
      }

      coreDetecting.value = true;
      try {
        const result = await serverApi.parseServerCoreType(path);
        if (requestId !== coreDetectRequestId) {
          return;
        }
        detectedCoreType.value = result.coreType;
        detectedCoreMainClass.value = result.mainClass ?? "";
      } catch (error) {
        if (requestId !== coreDetectRequestId) {
          return;
        }
        detectedCoreType.value = i18n.t("create.source_core_unknown");
        detectedCoreMainClass.value = "";
        showError(String(error));
      } finally {
        if (requestId === coreDetectRequestId) {
          coreDetecting.value = false;
        }
      }
    },
    { immediate: true },
  );

  watch(
    [sourcePath, sourceType],
    async ([path, type]) => {
      const requestId = ++startupDetectRequestId;

      if (!path.trim() || !type) {
        startupDetecting.value = false;
        startupCandidates.value = [];
        selectedStartupId.value = "";
        customStartupCommand.value = "";
        return;
      }

      startupDetecting.value = true;
      try {
        const discovered =
          type === "folder"
            ? await buildFolderStartupCandidates(path, serverApi.parseServerCoreType, isWindows)
            : await buildArchiveStartupCandidates(path, serverApi.parseServerCoreType);
        const list = appendCustomCandidate(discovered);

        if (requestId !== startupDetectRequestId) {
          return;
        }

        startupCandidates.value = list;
        if (!list.some((item) => item.id === selectedStartupId.value)) {
          selectedStartupId.value = list[0]?.id ?? "";
        }
      } catch (error) {
        if (requestId !== startupDetectRequestId) {
          return;
        }
        startupCandidates.value = appendCustomCandidate([]);
        selectedStartupId.value = startupCandidates.value[0]?.id ?? "";
        showError(String(error));
      } finally {
        if (requestId === startupDetectRequestId) {
          startupDetecting.value = false;
        }
      }
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
      runPath.value = selected;
    }
  }

  async function rescanStartupCandidates() {
    const path = sourcePath.value.trim();
    const type = sourceType.value;
    if (!path || !type) {
      startupCandidates.value = [];
      selectedStartupId.value = "";
      return;
    }

    const requestId = ++startupDetectRequestId;
    startupDetecting.value = true;
    try {
      const discovered =
        type === "folder"
          ? await buildFolderStartupCandidates(path, serverApi.parseServerCoreType, isWindows)
          : await buildArchiveStartupCandidates(path, serverApi.parseServerCoreType);
      const list = appendCustomCandidate(discovered);

      if (requestId !== startupDetectRequestId) {
        return;
      }

      startupCandidates.value = list;
      selectedStartupId.value = list[0]?.id ?? "";
    } catch (error) {
      if (requestId !== startupDetectRequestId) {
        return;
      }
      showError(String(error));
    } finally {
      if (requestId === startupDetectRequestId) {
        startupDetecting.value = false;
      }
    }
  }

  function validateBeforeSubmit(): boolean {
    clearError();

    if (!hasSource.value) {
      showError(i18n.t("create.source_required"));
      return false;
    }
    if (requiresRunPath.value && runPath.value.trim().length === 0) {
      showError(i18n.t("create.path_required_archive"));
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
      showError(i18n.t("create.startup_custom_not_supported"));
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

  function requestCopyConflictConfirm(conflicts: string[]): Promise<boolean> {
    copyConflictItems.value = conflicts;
    copyConflictDialogOpen.value = true;
    return new Promise<boolean>((resolve) => {
      copyConflictResolver = resolve;
    });
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
      if (sourceType.value === "archive") {
        await serverApi.importModpack({
          name: serverName.value.trim(),
          modpackPath: sourcePath.value,
          javaPath: selectedJava.value,
          maxMemory: parseNumber(maxMemory.value, 2048),
          minMemory: parseNumber(minMemory.value, 512),
          port: parseNumber(port.value, 25565),
        });
      } else {
        const sourceDir = sourcePath.value.trim();
        const targetDir = resolvedFolderServerPath.value;
        if (!targetDir) {
          throw new Error(i18n.t("create.path_required_folder_target"));
        }

        if (normalizePathForCompare(sourceDir) !== normalizePathForCompare(targetDir)) {
          const conflicts = await collectCopyConflicts(sourceDir, targetDir);
          if (conflicts.length > 0) {
            const confirmed = await requestCopyConflictConfirm(conflicts);
            if (!confirmed) {
              return;
            }
          }
          await copyDirectoryRecursive(sourceDir, targetDir);
        }

        const startup = selectedStartup.value;
        const startupMode = mapStartupModeForApi(startup?.mode ?? "jar");
        const executablePath = startup?.path
          ? resolveExecutablePathForTarget(startup.path, sourceDir, targetDir)
          : undefined;

        await serverApi.addExistingServer({
          name: serverName.value.trim(),
          serverPath: targetDir,
          javaPath: selectedJava.value,
          maxMemory: parseNumber(maxMemory.value, 2048),
          minMemory: parseNumber(minMemory.value, 512),
          port: parseNumber(port.value, 25565),
          startupMode,
          executablePath,
        });
      }

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
    coreDetecting,
    detectedCoreType,
    detectedCoreMainClass,
    startupDetecting,
    startupCandidates,
    selectedStartupId,
    customStartupCommand,
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
    showSourceCoreInfo,
    sourceCoreInfoText,
    pickRunPath,
    rescanStartupCandidates,
    detectJava,
    handleSubmit,
    confirmCopyConflict,
    cancelCopyConflict,
  };
}
