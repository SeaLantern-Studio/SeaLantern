import { onMounted, ref, type Ref } from "vue";
import type { JavaInfo } from "@api/java";
import { systemApi } from "@api/system";
import { useSettingsStore } from "@stores/settingsStore";
import { isBrowserEnv } from "@api/tauri";

interface UseCreateServerDefaultsOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<"archive" | "folder" | "">;
  runPath: Ref<string>;
  maxMemory: Ref<string>;
  minMemory: Ref<string>;
  port: Ref<string>;
  selectedJava: Ref<string>;
  javaList: Ref<JavaInfo[]>;
  isChildPathBlocked: (targetPath: string) => boolean;
  onInvalidRunPath: () => void;
}

export function useCreateServerDefaults(options: UseCreateServerDefaultsOptions) {
  const settingsStore = useSettingsStore();
  const loadingDefaults = ref(false);

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

  async function loadDefaultSettings() {
    loadingDefaults.value = true;
    try {
      const defaults = await systemApi.getCreateServerDefaults();

      options.maxMemory.value = String(defaults.default_max_memory);
      options.minMemory.value = String(defaults.default_min_memory);
      options.port.value = String(defaults.default_port);
      options.runPath.value = defaults.suggested_run_path || defaults.default_run_path;
      options.javaList.value = defaults.cached_java_list || [];
      options.selectedJava.value = defaults.preferred_java_path || "";
    } catch (error) {
      console.error("Failed to load default settings:", error);
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

  return {
    loadingDefaults,
    loadDefaultSettings,
    pickRunPath,
    applyRunPath,
    persistLastRunPath,
  };
}
