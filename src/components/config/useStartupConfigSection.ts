import { onUnmounted, ref, watch, type Ref } from "vue";
import { configApi, type SLStartupConfig } from "@api/config";
import { serverApi } from "@api/server";
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
  const jvmArgsText = ref("");
  const jvmPreset = ref<JvmPresetConfig>(createDefaultJvmPreset());
  const loading = ref(false);
  const saving = ref(false);
  const error = ref<string | null>(null);
  const cpuPolicy = ref(createDefaultCpuPolicy());
  const launchDetail = ref<StartupLaunchDetail | null>(null);
  const launchDetailLoading = ref(false);
  const launchDetailError = ref<string | null>(null);

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
    try {
      const config = await configApi.readSLConfig(options.serverPath.value);
      maxMemory.value = config.max_memory ?? options.defaultMaxMemory.value;
      minMemory.value = config.min_memory ?? options.defaultMinMemory.value;
      jvmArgsText.value = deserializeJvmArgs(config.jvm_args);
      jvmPreset.value = normalizeJvmPreset(config.jvm_preset);
      cpuPolicy.value = normalizeCpuPolicy(config.cpu_policy);
    } catch (e: any) {
      error.value = e?.toString() || "加载启动配置失败";
    } finally {
      loading.value = false;
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
      launchDetailError.value = e?.toString() || "加载真实启动详情失败";
    } finally {
      if (currentToken === launchDetailRequestToken) {
        launchDetailLoading.value = false;
      }
    }
  }

  async function saveConfig() {
    if (!options.serverPath.value || saving.value) return;
    if (maxMemory.value < 128) {
      error.value = "最大内存不能小于 128MB";
      return;
    }
    if (minMemory.value < 128) {
      error.value = "最小内存不能小于 128MB";
      return;
    }
    if (minMemory.value > maxMemory.value) {
      error.value = "最小内存不能大于最大内存";
      return;
    }

    const cpuPolicyError = getCpuPolicyValidationError(cpuPolicy.value);
    if (cpuPolicyError) {
      error.value = i18n.t(`config.cpu_policy_invalid_${cpuPolicyError}`);
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
      options.onSaved?.(maxMemory.value, minMemory.value);
      void loadLaunchDetail();
    } catch (e: any) {
      error.value = e?.toString() || "保存启动配置失败";
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

  watch([maxMemory, minMemory, jvmArgsText, jvmPreset, cpuPolicy], () => {
    scheduleAutoSave();
  });

  return {
    maxMemory,
    minMemory,
    jvmArgsText,
    jvmPreset,
    cpuPolicy,
    loading,
    saving,
    error,
    launchDetail,
    launchDetailLoading,
    launchDetailError,
    loadConfig,
    loadLaunchDetail,
    saveConfig,
  };
}
