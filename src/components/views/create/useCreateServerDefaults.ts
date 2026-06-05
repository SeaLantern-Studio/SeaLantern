import { onActivated, onMounted, ref, type Ref } from "vue";
import type { JavaInfo } from "@api/java";
import { systemApi } from "@api/system";
import { useSettingsStore } from "@stores/settingsStore";
import { isBrowserEnv } from "@api/tauri";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";
import {
  createDefaultCpuPolicy,
  createDefaultJvmPreset,
  deserializeJvmArgs,
  normalizeCpuPolicy,
  normalizeJvmPreset,
} from "@utils/serverStartupConfig";

interface UseCreateServerDefaultsOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<"archive" | "folder" | "">;
  runPath: Ref<string>;
  maxMemory: Ref<string>;
  minMemory: Ref<string>;
  port: Ref<string>;
  selectedJava: Ref<string>;
  javaList: Ref<JavaInfo[]>;
  jvmArgsText: Ref<string>;
  jvmPreset: Ref<JvmPresetConfig>;
  cpuPolicy: Ref<CpuPolicyConfig>;
  isChildPathBlocked: (targetPath: string) => boolean;
  onInvalidRunPath: () => void;
}

function sameJvmPreset(left: JvmPresetConfig, right: JvmPresetConfig): boolean {
  return left.preset === right.preset;
}

function sameCpuPolicy(left: CpuPolicyConfig, right: CpuPolicyConfig): boolean {
  return (
    left.mode === right.mode &&
    (left.count ?? null) === (right.count ?? null) &&
    (left.explicit_set ?? null) === (right.explicit_set ?? null) &&
    left.sync_active_processor_count === right.sync_active_processor_count
  );
}

export function useCreateServerDefaults(options: UseCreateServerDefaultsOptions) {
  const settingsStore = useSettingsStore();
  const loadingDefaults = ref(false);
  const lastAppliedJvmArgsText = ref("");
  const lastAppliedJvmPreset = ref<JvmPresetConfig>(createDefaultJvmPreset());
  const lastAppliedCpuPolicy = ref<CpuPolicyConfig>(createDefaultCpuPolicy());
  const advancedDefaultsInitialized = ref(false);

  function syncAdvancedDefaults(
    nextJvmArgsText: string,
    nextJvmPreset: JvmPresetConfig,
    nextCpuPolicy: CpuPolicyConfig,
    force: boolean,
  ) {
    if (
      force ||
      !advancedDefaultsInitialized.value ||
      options.jvmArgsText.value === lastAppliedJvmArgsText.value
    ) {
      options.jvmArgsText.value = nextJvmArgsText;
    }

    if (
      force ||
      !advancedDefaultsInitialized.value ||
      sameJvmPreset(options.jvmPreset.value, lastAppliedJvmPreset.value)
    ) {
      options.jvmPreset.value = nextJvmPreset;
    }

    if (
      force ||
      !advancedDefaultsInitialized.value ||
      sameCpuPolicy(options.cpuPolicy.value, lastAppliedCpuPolicy.value)
    ) {
      options.cpuPolicy.value = nextCpuPolicy;
    }

    lastAppliedJvmArgsText.value = nextJvmArgsText;
    lastAppliedJvmPreset.value = nextJvmPreset;
    lastAppliedCpuPolicy.value = nextCpuPolicy;
    advancedDefaultsInitialized.value = true;
  }

  function applyRunPath(nextPath: string): boolean {
    const targetPath = nextPath.trim();
    if (options.sourceType.value === "folder" && options.isChildPathBlocked(targetPath)) {
      options.onInvalidRunPath();
      return false;
    }

    options.runPath.value = nextPath;
    return true;
  }

  async function persistLastRunPath(nextPath: string) {
    try {
      await settingsStore.updatePartial({ last_run_path: nextPath });
    } catch (error) {
      console.error("Failed to save last run path:", error);
    }
  }

  async function loadDefaultSettings(syncAllFields = true) {
    loadingDefaults.value = true;
    try {
      const defaults = await systemApi.getCreateServerDefaults();

      if (syncAllFields) {
        options.maxMemory.value = String(defaults.default_max_memory);
        options.minMemory.value = String(defaults.default_min_memory);
        options.port.value = String(defaults.default_port);
        options.runPath.value = defaults.suggested_run_path || defaults.default_run_path;
        options.javaList.value = defaults.cached_java_list || [];
        options.selectedJava.value = defaults.preferred_java_path || "";
      }

      syncAdvancedDefaults(
        deserializeJvmArgs(defaults.default_jvm_args),
        normalizeJvmPreset(defaults.default_jvm_preset),
        normalizeCpuPolicy(defaults.default_cpu_policy),
        syncAllFields,
      );
    } catch (error) {
      console.error("Failed to load default settings:", error);
      if (!advancedDefaultsInitialized.value) {
        syncAdvancedDefaults("", createDefaultJvmPreset(), createDefaultCpuPolicy(), true);
      }
    } finally {
      loadingDefaults.value = false;
    }
  }

  async function pickRunPath() {
    if (isBrowserEnv()) {
      try {
        const defaults = await systemApi.getCreateServerDefaults();
        const fullPath = defaults.suggested_run_path || defaults.default_run_path;
        if (applyRunPath(fullPath)) {
          await persistLastRunPath(fullPath);
        }
      } catch (error) {
        console.error("Failed to get create server defaults:", error);
      }
      return;
    }

    const selected = await systemApi.pickFolder();
    if (!selected) {
      return;
    }

    if (applyRunPath(selected)) {
      await persistLastRunPath(selected);
    }
  }

  onMounted(() => {
    void loadDefaultSettings();
  });

  onActivated(() => {
    void loadDefaultSettings(false);
  });

  return {
    loadingDefaults,
    loadDefaultSettings,
    pickRunPath,
    applyRunPath,
    persistLastRunPath,
  };
}
