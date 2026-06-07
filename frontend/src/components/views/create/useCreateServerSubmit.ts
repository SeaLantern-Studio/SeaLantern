import { computed, type Ref } from "vue";
import { useRouter } from "vue-router";
import { serverApi } from "@api/server";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";
import {
  containsIoRedirection,
  isStrictChildPath,
  mapStartupModeForModpack,
} from "@components/views/create/startupUtils";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import {
  getCpuPolicyValidationError,
  normalizeCpuPolicy,
  normalizeJvmPreset,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";

type SourceType = "archive" | "folder" | "";

interface UseCreateServerSubmitOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<SourceType>;
  runPath: Ref<string>;
  startupSyncPending: Ref<boolean>;
  startupDetecting: Ref<boolean>;
  startupCandidates: Ref<StartupCandidate[]>;
  selectedStartupId: Ref<string>;
  customStartupCommand: Ref<string>;
  detectedCoreTypeKey: Ref<string>;
  selectedCoreType: Ref<string>;
  detectedMcVersion: Ref<string>;
  selectedMcVersion: Ref<string>;
  mcVersionDetectionFailed: Ref<boolean>;
  serverName: Ref<string>;
  maxMemory: Ref<string>;
  minMemory: Ref<string>;
  port: Ref<string>;
  selectedJava: Ref<string>;
  onlineMode: Ref<boolean>;
  jvmArgsText: Ref<string>;
  jvmPreset: Ref<JvmPresetConfig>;
  cpuPolicy: Ref<CpuPolicyConfig>;
  startCreating: () => void;
  stopCreating: () => void;
  showError: (message: string) => void;
  clearError: () => void;
}

function parseNumber(value: string, fallbackValue: number): number {
  const parsed = Number.parseInt(value, 10);
  return Number.isNaN(parsed) ? fallbackValue : parsed;
}

export function useCreateServerSubmit(options: UseCreateServerSubmitOptions) {
  const router = useRouter();
  const serverStore = useServerStore();

  const selectedStartup = computed(
    () =>
      options.startupCandidates.value.find((item) => item.id === options.selectedStartupId.value) ??
      null,
  );

  const starterSelected = computed(() => selectedStartup.value?.mode === "starter");
  const customCommandHasRedirect = computed(
    () =>
      selectedStartup.value?.mode === "custom" &&
      containsIoRedirection(options.customStartupCommand.value),
  );
  const selectedCustomStartupPath = computed(() => {
    if (selectedStartup.value?.mode !== "custom") {
      return "";
    }
    return selectedStartup.value.path.trim();
  });
  const hasExplicitCustomCommand = computed(
    () => options.customStartupCommand.value.trim().length > 0,
  );
  const customStartupNeedsManualCommand = computed(
    () => selectedStartup.value?.mode === "custom" && selectedCustomStartupPath.value.length === 0,
  );

  const hasSource = computed(
    () => options.sourcePath.value.trim().length > 0 && options.sourceType.value !== "",
  );
  const hasPathStep = computed(() => hasSource.value && options.runPath.value.trim().length > 0);
  const hasStartupStep = computed(() => {
    if (!hasPathStep.value || !selectedStartup.value) {
      return false;
    }

    if (selectedStartup.value.mode === "custom") {
      if (customCommandHasRedirect.value) {
        return false;
      }

      return hasExplicitCustomCommand.value || selectedCustomStartupPath.value.length > 0;
    }

    return !(
      selectedStartup.value.mode === "starter" &&
      options.mcVersionDetectionFailed.value &&
      options.selectedMcVersion.value.trim().length === 0
    );
  });
  const requiresJava = computed(() => selectedStartup.value?.mode !== "custom");
  const hasJava = computed(
    () => !requiresJava.value || options.selectedJava.value.trim().length > 0,
  );
  const hasServerConfig = computed(() => options.serverName.value.trim().length > 0);

  const step1Completed = computed(() => hasSource.value);
  const step2Completed = computed(() => step1Completed.value && hasPathStep.value);
  const step3Completed = computed(() => step2Completed.value && hasStartupStep.value);
  const step4Completed = computed(
    () => step3Completed.value && hasJava.value && hasServerConfig.value,
  );

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

  const canSubmit = computed(
    () =>
      step4Completed.value && !options.startupSyncPending.value && !options.startupDetecting.value,
  );

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

  function validateBeforeSubmit(): boolean {
    options.clearError();

    if (!hasSource.value) {
      options.showError(i18n.t("create.source_required"));
      return false;
    }
    if (options.runPath.value.trim().length === 0) {
      options.showError(i18n.t("create.path_required"));
      return false;
    }
    if (
      options.sourceType.value === "folder" &&
      isStrictChildPath(options.runPath.value, options.sourcePath.value)
    ) {
      options.showError(i18n.t("create.path_child_of_source_forbidden"));
      return false;
    }
    if (!selectedStartup.value) {
      options.showError(i18n.t("create.startup_required"));
      return false;
    }

    if (selectedStartup.value.mode === "custom") {
      if (customStartupNeedsManualCommand.value && !hasExplicitCustomCommand.value) {
        options.showError(i18n.t("create.startup_custom_required"));
        return false;
      }
      if (containsIoRedirection(options.customStartupCommand.value)) {
        options.showError(i18n.t("create.startup_custom_redirect_forbidden"));
        return false;
      }
    }

    if (
      selectedStartup.value.mode === "starter" &&
      options.mcVersionDetectionFailed.value &&
      options.selectedMcVersion.value.trim().length === 0
    ) {
      options.showError(i18n.t("create.startup_mc_version_required"));
      return false;
    }

    if (requiresJava.value && !options.selectedJava.value) {
      options.showError(i18n.t("common.select_java_path"));
      return false;
    }
    if (!options.serverName.value.trim()) {
      options.showError(i18n.t("common.enter_server_name"));
      return false;
    }

    const cpuPolicyError = getCpuPolicyValidationError(options.cpuPolicy.value);
    if (cpuPolicyError) {
      options.showError(i18n.t(`create.cpu_policy_invalid_${cpuPolicyError}`));
      return false;
    }

    return true;
  }

  async function handleSubmit() {
    if (!validateBeforeSubmit()) {
      return;
    }

    options.startCreating();
    try {
      const startup = selectedStartup.value;
      const startupMode = mapStartupModeForModpack(startup?.mode ?? "jar");
      const resolvedCoreType =
        options.selectedCoreType.value.trim() || options.detectedCoreTypeKey.value.trim();
      const resolvedMcVersion =
        startupMode === "starter"
          ? options.selectedMcVersion.value.trim() || options.detectedMcVersion.value.trim()
          : "";

      const createdServer = await serverApi.importModpack({
        name: options.serverName.value.trim(),
        modpackPath: options.sourcePath.value,
        javaPath: requiresJava.value ? options.selectedJava.value : "",
        maxMemory: parseNumber(options.maxMemory.value, 2048),
        minMemory: parseNumber(options.minMemory.value, 512),
        port: parseNumber(options.port.value, 25565),
        startupMode,
        onlineMode: options.onlineMode.value,
        customCommand:
          startupMode === "custom" && hasExplicitCustomCommand.value
            ? options.customStartupCommand.value.trim()
            : undefined,
        runPath: options.runPath.value.trim(),
        startupFilePath: startup?.path || undefined,
        coreType: resolvedCoreType || undefined,
        mcVersion: resolvedMcVersion || undefined,
        jvmArgs: serializeJvmArgsText(options.jvmArgsText.value),
        cpuPolicy: normalizeCpuPolicy(options.cpuPolicy.value),
        jvmPreset: normalizeJvmPreset(options.jvmPreset.value),
      });

      await serverStore.refreshList();
      serverStore.setCurrentServer(createdServer.id);

      try {
        await serverApi.start(createdServer.id);
        await serverStore.refreshStatus(createdServer.id);
      } catch (error) {
        await serverStore.refreshStatus(createdServer.id);
        options.showError(
          i18n.t("create.created_but_start_failed", {
            name: createdServer.name,
            error: String(error),
          }),
        );
        router.push(`/console/${createdServer.id}`);
        return;
      }

      router.push(`/console/${createdServer.id}`);
    } catch (error) {
      options.showError(String(error));
    } finally {
      options.stopCreating();
    }
  }

  return {
    selectedStartup,
    starterSelected,
    customCommandHasRedirect,
    activeStep,
    stepItems,
    canSubmit,
    handleSubmit,
  };
}
