import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import { clearLogs, exportAppLogs, getLogs, type LogLine } from "@api/logging";
import { systemApi, type SystemInfo } from "@api/system";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";

const LOG_POLL_INTERVAL = 1500;
const SYSTEM_POLL_INTERVAL = 5000;
const LOG_LIMIT = 300;

interface UseDeveloperToolsOptions {
  enabled: () => boolean;
}

export function useDeveloperTools(options: UseDeveloperToolsOptions) {
  const globalMessage = useGlobalMessage();

  const logs = ref<LogLine[]>([]);
  const systemInfo = ref<SystemInfo | null>(null);
  const version = ref("-");
  const loadingLogs = ref(false);
  const loadingSystem = ref(false);
  const exportingLogs = ref(false);
  const clearingLogs = ref(false);
  const downloadingUpdate = ref(false);
  const triggeringCrash = ref(false);
  const updateUrl = ref("");
  const logError = ref<string | null>(null);
  const systemError = ref<string | null>(null);

  let logTimer: ReturnType<typeof setInterval> | null = null;
  let systemTimer: ReturnType<typeof setInterval> | null = null;

  const logText = computed(() => logs.value.map((entry) => entry.formatted).join("\n"));
  const logLines = computed(() => logs.value.map((entry) => entry.formatted));
  const systemSummary = computed(() => {
    if (!systemInfo.value) return "";

    return [
      `${i18n.t("developer.app_version")}: ${version.value}`,
      `${i18n.t("developer.os")}: ${systemInfo.value.os_name} ${systemInfo.value.os_version}`,
      `${i18n.t("developer.kernel")}: ${systemInfo.value.kernel_version}`,
      `${i18n.t("developer.host")}: ${systemInfo.value.host_name || "-"}`,
      `${i18n.t("developer.cpu")}: ${systemInfo.value.cpu.name} (${systemInfo.value.cpu.count})`,
      `${i18n.t("developer.memory")}: ${Math.round(systemInfo.value.memory.used / 1024 / 1024 / 1024)} / ${Math.round(systemInfo.value.memory.total / 1024 / 1024 / 1024)} GB`,
      `${i18n.t("developer.process_count")}: ${systemInfo.value.process_count}`,
      `${i18n.t("developer.uptime")}: ${Math.floor(systemInfo.value.uptime / 60)} min`,
    ].join("\n");
  });
  const isBrowserMode = computed(() => isBrowserEnv());
  const canTriggerCrash = computed(() => !isBrowserMode.value && import.meta.env.DEV);
  const isEnabled = computed(() => options.enabled());

  async function loadVersion() {
    if (isBrowserMode.value) {
      version.value = import.meta.env.VITE_APP_VERSION || "Web";
      return;
    }

    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      version.value = await getVersion();
    } catch {
      version.value = import.meta.env.VITE_APP_VERSION || "-";
    }
  }

  async function refreshLogs() {
    if (!isEnabled.value) {
      logs.value = [];
      logError.value = null;
      return;
    }

    if (loadingLogs.value) return;

    loadingLogs.value = true;
    try {
      logs.value = await getLogs(LOG_LIMIT);
      logError.value = null;
    } catch (error) {
      logError.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingLogs.value = false;
    }
  }

  async function refreshSystemInfo() {
    if (!isEnabled.value) {
      systemInfo.value = null;
      systemError.value = null;
      return;
    }

    if (loadingSystem.value) return;

    loadingSystem.value = true;
    try {
      systemInfo.value = await systemApi.getSystemInfo();
      systemError.value = null;
    } catch (error) {
      systemError.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingSystem.value = false;
    }
  }

  async function copyLogs() {
    try {
      await navigator.clipboard.writeText(logText.value);
      globalMessage.success(i18n.t("developer.logs_copied", { count: logs.value.length }));
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function exportLogsToFile() {
    if (exportingLogs.value || logs.value.length === 0) return;

    exportingLogs.value = true;
    try {
      const savePath = await systemApi.pickSaveFile();
      if (!savePath) return;

      await exportAppLogs(savePath);
      globalMessage.success(i18n.t("developer.logs_exported"));
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    } finally {
      exportingLogs.value = false;
    }
  }

  async function clearAllLogs() {
    if (clearingLogs.value) return;

    clearingLogs.value = true;
    try {
      await clearLogs();
      logs.value = [];
      globalMessage.success(i18n.t("developer.logs_cleared"));
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    } finally {
      clearingLogs.value = false;
    }
  }

  async function copySystemSummary() {
    try {
      await navigator.clipboard.writeText(systemSummary.value);
      globalMessage.success(i18n.t("developer.system_copied"));
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function downloadUpdateFromUrl() {
    const url = updateUrl.value.trim();
    if (!url || downloadingUpdate.value) return;

    downloadingUpdate.value = true;
    try {
      await tauriInvoke("download_update_from_debug_url", { url });
      globalMessage.success(i18n.t("developer.update_test_started"));
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    } finally {
      downloadingUpdate.value = false;
    }
  }

  async function triggerCrashTest() {
    if (!canTriggerCrash.value || triggeringCrash.value) return;

    triggeringCrash.value = true;
    try {
      await tauriInvoke("debug_panic");
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
      triggeringCrash.value = false;
    }
  }

  function startPolling() {
    if (!isEnabled.value) return;
    stopPolling();

    void refreshLogs();
    void refreshSystemInfo();

    logTimer = setInterval(() => {
      void refreshLogs();
    }, LOG_POLL_INTERVAL);

    systemTimer = setInterval(() => {
      void refreshSystemInfo();
    }, SYSTEM_POLL_INTERVAL);
  }

  function stopPolling() {
    if (logTimer) {
      clearInterval(logTimer);
      logTimer = null;
    }
    if (systemTimer) {
      clearInterval(systemTimer);
      systemTimer = null;
    }
  }

  onMounted(() => {
    void loadVersion();
    if (isEnabled.value) {
      startPolling();
    }
  });

  watch(isEnabled, (enabled) => {
    if (enabled) {
      startPolling();
      return;
    }

    stopPolling();
    logs.value = [];
    systemInfo.value = null;
    logError.value = null;
    systemError.value = null;
  });

  onUnmounted(() => {
    stopPolling();
  });

  return {
    logs,
    systemInfo,
    version,
    loadingLogs,
    loadingSystem,
    exportingLogs,
    clearingLogs,
    downloadingUpdate,
    triggeringCrash,
    updateUrl,
    logError,
    systemError,
    logText,
    logLines,
    systemSummary,
    isBrowserMode,
    canTriggerCrash,
    refreshLogs,
    refreshSystemInfo,
    copyLogs,
    exportLogsToFile,
    clearAllLogs,
    copySystemSummary,
    downloadUpdateFromUrl,
    triggerCrashTest,
  };
}
