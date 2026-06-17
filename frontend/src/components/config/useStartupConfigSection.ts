import { onUnmounted, ref, watch, type Ref } from "vue";
import { configApi, type SLStartupConfig } from "@api/config";
import { javaApi, type JavaInfo } from "@api/java";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import type {
  DockerLaunchDetail,
  JvmPresetConfig,
  LocalLaunchDetail,
  ServerRuntimeKind,
} from "@type/server";
import {
  createDefaultCpuPolicy,
  createDefaultJvmPreset,
  deserializeJvmArgs,
  getCpuPolicyValidationError,
  normalizeCpuPolicy,
  normalizeJvmPreset,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";

interface UseStartupConfigSectionOptions {
  serverPath: Ref<string>;
  serverId: Ref<string | null>;
  runtimeKind: Ref<ServerRuntimeKind | null>;
  defaultMaxMemory: Ref<number>;
  defaultMinMemory: Ref<number>;
  onSaved?: (maxMemory: number, minMemory: number) => void;
}

type StartupLaunchDetail =
  | { kind: "local"; detail: LocalLaunchDetail }
  | { kind: "docker_itzg"; detail: DockerLaunchDetail };

const AUTO_SAVE_DELAY = 800;

export function useStartupConfigSection(options: UseStartupConfigSectionOptions) {
  const maxMemory = ref(options.defaultMaxMemory.value);
  const minMemory = ref(options.defaultMinMemory.value);
  const selectedJava = ref("");
  const javaList = ref<JavaInfo[]>([]);
  const jvmArgsText = ref("");
  const jvmPreset = ref<JvmPresetConfig>(createDefaultJvmPreset());
  const loading = ref(false);
  const javaLoading = ref(false);
  const saving = ref(false);
  const error = ref<string | null>(null);
  const cpuPolicy = ref(createDefaultCpuPolicy());
  const launchDetail = ref<StartupLaunchDetail | null>(null);
  const launchDetailLoading = ref(false);
  const launchDetailError = ref<string | null>(null);
  const configLoadWarning = ref<string | null>(null);

  let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
  let launchDetailRequestToken = 0;

  function clearAutoSaveTimer() {
    if (autoSaveTimer) {
      clearTimeout(autoSaveTimer);
      autoSaveTimer = null;
    }
  }

  function scheduleAutoSave() {
    clearAutoSaveTimer();
    autoSaveTimer = setTimeout(() => {
      void saveConfig();
    }, AUTO_SAVE_DELAY);
  }

  async function loadConfig() {
    if (!options.serverPath.value) return;
    loading.value = true;
    error.value = null;
    configLoadWarning.value = null;
    try {
      const config = await configApi.readSLConfig(options.serverPath.value);
      maxMemory.value = config.max_memory ?? options.defaultMaxMemory.value;
      minMemory.value = config.min_memory ?? options.defaultMinMemory.value;
      jvmArgsText.value = deserializeJvmArgs(config.jvm_args);
      jvmPreset.value = normalizeJvmPreset(config.jvm_preset);
      cpuPolicy.value = normalizeCpuPolicy(config.cpu_policy);

      const serverId = options.serverId.value;
      if (serverId) {
        try {
          const servers = await serverApi.getList();
          const currentServer = servers.find((server) => server.id === serverId);
          selectedJava.value =
            currentServer?.runtime.kind === "local" ? currentServer.runtime.java_path : "";
        } catch (e: any) {
          configLoadWarning.value =
            e?.toString() || i18n.t("config.startup_config_java_path_load_failed");
        }
      }
    } catch (e: any) {
      error.value = e?.toString() || i18n.t("config.startup_config_load_failed");
    } finally {
      loading.value = false;
    }
  }

  async function loadJavaChoices() {
    javaLoading.value = true;
    try {
      const defaults = await systemApi.getCreateServerDefaults();
      javaList.value = defaults.cached_java_list || [];
      if (!selectedJava.value.trim()) {
        selectedJava.value = defaults.preferred_java_path || "";
      }
    } catch (e: any) {
      error.value = e?.toString() || i18n.t("config.java_list_load_failed");
    } finally {
      javaLoading.value = false;
    }
  }

  async function detectJava() {
    javaLoading.value = true;
    error.value = null;
    try {
      const detected = await javaApi.detect();
      javaList.value = detected;

      if (!selectedJava.value.trim() && detected.length > 0) {
        const preferredJava = detected.find((java) => java.is_64bit && java.major_version >= 17);
        selectedJava.value = preferredJava ? preferredJava.path : detected[0].path;
      }
    } catch (e: any) {
      error.value = e?.toString() || i18n.t("config.java_scan_failed");
    } finally {
      javaLoading.value = false;
    }
  }

  async function loadLaunchDetail() {
    const serverId = options.serverId.value;
    const runtimeKind = options.runtimeKind.value;

    launchDetailRequestToken += 1;
    const currentToken = launchDetailRequestToken;

    if (!serverId || !runtimeKind) {
      launchDetail.value = null;
      launchDetailError.value = null;
      launchDetailLoading.value = false;
      return;
    }

    launchDetailLoading.value = true;
    launchDetailError.value = null;

    try {
      if (runtimeKind === "local") {
        const detail = await serverApi.getLocalLaunchDetail(serverId);
        if (currentToken !== launchDetailRequestToken) {
          return;
        }
        launchDetail.value = { kind: "local", detail };
        return;
      }

      const detail = await serverApi.getDockerLaunchDetail(serverId);
      if (currentToken !== launchDetailRequestToken) {
        return;
      }
      launchDetail.value = { kind: "docker_itzg", detail };
    } catch (e: any) {
      if (currentToken !== launchDetailRequestToken) {
        return;
      }
      launchDetail.value = null;
      launchDetailError.value = e?.toString() || i18n.t("config.startup_launch_detail_load_failed");
    } finally {
      if (currentToken === launchDetailRequestToken) {
        launchDetailLoading.value = false;
      }
    }
  }

  async function saveConfig() {
    if (!options.serverPath.value || saving.value) return;
    if (maxMemory.value < 128) {
      error.value = i18n.t("config.max_memory_too_small");
      return;
    }
    if (minMemory.value < 128) {
      error.value = i18n.t("config.min_memory_too_small");
      return;
    }
    if (minMemory.value > maxMemory.value) {
      error.value = i18n.t("config.min_memory_gt_max_memory");
      return;
    }

    const cpuPolicyError = getCpuPolicyValidationError(cpuPolicy.value);
    if (cpuPolicyError) {
      error.value = i18n.t(`config.cpu_policy_invalid_${cpuPolicyError}`);
      return;
    }
    if (options.runtimeKind.value === "local" && !selectedJava.value.trim()) {
      error.value = i18n.t("common.select_java_path");
      return;
    }

    saving.value = true;
    error.value = null;
    try {
      const config: SLStartupConfig = {
        max_memory: maxMemory.value,
        min_memory: minMemory.value,
        jvm_args: serializeJvmArgsText(jvmArgsText.value),
        cpu_policy: normalizeCpuPolicy(cpuPolicy.value),
        jvm_preset: normalizeJvmPreset(jvmPreset.value),
      };
      await configApi.writeSLConfig(options.serverPath.value, config);
      if (options.runtimeKind.value === "local") {
        const serverId = options.serverId.value;
        if (!serverId) {
          throw new Error(i18n.t("config.server_id_missing_for_java_path_save"));
        }
        await serverApi.updateServerJavaPath(serverId, selectedJava.value.trim());
      }
      options.onSaved?.(maxMemory.value, minMemory.value);
      void loadLaunchDetail();
    } catch (e: any) {
      error.value = e?.toString() || i18n.t("config.startup_config_save_failed");
    } finally {
      saving.value = false;
    }
  }

  onUnmounted(() => {
    clearAutoSaveTimer();
  });

  watch(
    options.serverPath,
    () => {
      void loadConfig();
      void loadJavaChoices();
    },
    { immediate: true },
  );

  watch(
    [options.serverId, options.runtimeKind],
    () => {
      void loadLaunchDetail();
    },
    { immediate: true },
  );

  watch([maxMemory, minMemory, selectedJava, jvmArgsText, jvmPreset, cpuPolicy], () => {
    scheduleAutoSave();
  });

  return {
    maxMemory,
    minMemory,
    selectedJava,
    javaList,
    jvmArgsText,
    jvmPreset,
    cpuPolicy,
    loading,
    javaLoading,
    saving,
    error,
    launchDetail,
    launchDetailLoading,
    launchDetailError,
    configLoadWarning,
    loadConfig,
    loadJavaChoices,
    detectJava,
    loadLaunchDetail,
    saveConfig,
  };
}
