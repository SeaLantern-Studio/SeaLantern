import { onMounted, onUnmounted, ref, watch, type Ref } from "vue";
import { configApi, type SLStartupConfig } from "@api/config";
import type { JvmPresetConfig } from "@type/server";
import {
  createDefaultCpuPolicy,
  createDefaultJvmPreset,
  deserializeJvmArgs,
  normalizeCpuPolicy,
  normalizeJvmPreset,
  serializeJvmArgsText,
} from "@utils/serverStartupConfig";

interface UseStartupConfigSectionOptions {
  serverPath: Ref<string>;
  defaultMaxMemory: Ref<number>;
  defaultMinMemory: Ref<number>;
  onSaved?: (maxMemory: number, minMemory: number) => void;
}

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

  let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;

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
    } catch (e: any) {
      error.value = e?.toString() || "保存启动配置失败";
    } finally {
      saving.value = false;
    }
  }

  onMounted(() => {
    void loadConfig();
  });

  onUnmounted(() => {
    clearAutoSaveTimer();
  });

  watch(options.serverPath, () => {
    void loadConfig();
  });

  watch([maxMemory, minMemory, jvmArgsText, jvmPreset], () => {
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
    loadConfig,
    saveConfig,
  };
}
