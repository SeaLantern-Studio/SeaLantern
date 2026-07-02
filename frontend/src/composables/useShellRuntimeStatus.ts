import { computed, onMounted, shallowRef } from "vue";
import { systemApi } from "@api/system";
import { isBrowserEnv } from "@api/tauri";
import type { UiShellId } from "@api/settings";
import { i18n } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";

interface UseShellRuntimeStatusOptions {
  logScope: string;
}

const DESKTOP_PRIMARY_SHELL: UiShellId = "next";

export function useShellRuntimeStatus(options: UseShellRuntimeStatusOptions) {
  const browserOnly = isBrowserEnv();
  const loading = shallowRef(!browserOnly);
  const safeMode = shallowRef(false);
  const errorMessage = shallowRef<string | null>(null);

  const currentShellId = computed<UiShellId>(() => DESKTOP_PRIMARY_SHELL);
  const currentShellName = computed(() => i18n.t(`plugins.ui_shell.shells.${currentShellId.value}.name`));

  async function refreshStatus(): Promise<void> {
    if (browserOnly) {
      loading.value = false;
      errorMessage.value = null;
      safeMode.value = false;
      return;
    }

    loading.value = true;
    errorMessage.value = null;

    try {
      safeMode.value = await systemApi.getSafeModeStatus();
    } catch (error) {
      const normalized = normalizeAppError(error);
      errorMessage.value = normalized.message;
      pluginLogger.error(options.logScope, "界面运行状态读取失败", normalized);
    } finally {
      loading.value = false;
    }
  }

  onMounted(() => {
    void refreshStatus();
  });

  return {
    browserOnly,
    loading,
    safeMode,
    errorMessage,
    currentShellId,
    currentShellName,
    refreshStatus,
  };
}
