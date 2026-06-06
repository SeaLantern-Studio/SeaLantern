import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import { clearLogs, exportAppLogs, getLogs, type LogLine } from "@api/logging";
import { systemApi, type SystemInfo } from "@api/system";
import { useGlobalMessage } from "@composables/useMessage";
import { useSerialPolling } from "@composables/useSerialPolling";
import { i18n } from "@language";
import { useSettingsStore } from "@stores/settingsStore";
import { formatBytes } from "@utils/formatters";

const LOG_POLL_INTERVAL = 1500;
const SYSTEM_POLL_INTERVAL = 5000;
const LOG_LIMIT = 300;

interface UseDeveloperToolsOptions {
  enabled: () => boolean;
}

interface LogFilterOption {
  label: string;
  value: string;
}

const ALL_LOG_LEVELS = "__all_levels__";
const ALL_LOG_MODULES = "__all_modules__";

export function useDeveloperTools(options: UseDeveloperToolsOptions) {
  const globalMessage = useGlobalMessage();
  const settingsStore = useSettingsStore();

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
  const selectedLogLevel = ref(ALL_LOG_LEVELS);
  const selectedLogModule = ref(ALL_LOG_MODULES);

  const logLevelOptions = computed<LogFilterOption[]>(() => {
    const levels = Array.from(new Set(logs.value.map((entry) => entry.level))).sort();
    return [
      { label: i18n.t("developer.all_log_levels"), value: ALL_LOG_LEVELS },
      ...levels.map((level) => ({ label: level, value: level })),
    ];
  });
  const logModuleOptions = computed<LogFilterOption[]>(() => {
    const modules = Array.from(new Set(logs.value.map((entry) => entry.module))).sort();
    return [
      { label: i18n.t("developer.all_log_modules"), value: ALL_LOG_MODULES },
      ...modules.map((module) => ({ label: module, value: module })),
    ];
  });
  const filteredLogs = computed(() => {
    return logs.value.filter((entry) => {
      const levelMatches =
        selectedLogLevel.value === ALL_LOG_LEVELS || entry.level === selectedLogLevel.value;
      const moduleMatches =
        selectedLogModule.value === ALL_LOG_MODULES || entry.module === selectedLogModule.value;
      return levelMatches && moduleMatches;
    });
  });
  const logText = computed(() => filteredLogs.value.map((entry) => entry.formatted).join("\n"));
  const logLines = computed(() => filteredLogs.value.map((entry) => entry.formatted));
  const filteredLogCount = computed(() => filteredLogs.value.length);
  const totalLogCount = computed(() => logs.value.length);
  const hasLogEntries = computed(() => logs.value.length > 0);
  const memoryDisplayPrecision = computed(() => {
    const value = settingsStore.settings.memory_display_precision;
    return value === 0 || value === 2 || value === 4 ? value : 2;
  });

  function formatBytesToGb(value: number): string {
    const gb = value / 1024 / 1024 / 1024;
    return `${gb.toFixed(memoryDisplayPrecision.value)} GB`;
  }

  function formatBytesWithPrecision(value: number): string {
    if (value <= 0) {
      return formatBytes(0);
    }

    const units = ["B", "KB", "MB", "GB", "TB", "PB"];
    const base = 1024;
    const unitIndex = Math.min(Math.floor(Math.log(value) / Math.log(base)), units.length - 1);
    const scaled = value / Math.pow(base, unitIndex);
    return `${scaled.toFixed(memoryDisplayPrecision.value)} ${units[unitIndex]}`;
  }

  function formatUptime(totalSeconds: number): string {
    const days = Math.floor(totalSeconds / 86400);
    const hours = Math.floor((totalSeconds % 86400) / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const parts: string[] = [];

    if (days > 0) {
      parts.push(`${days}d`);
    }
    if (hours > 0 || days > 0) {
      parts.push(`${hours}h`);
    }
    parts.push(`${minutes}m`);
    return parts.join(" ");
  }

  const systemSummary = computed(() => {
    if (!systemInfo.value) return "";

    return [
      `${i18n.t("developer.app_version")}: ${version.value}`,
      `${i18n.t("developer.os")}: ${systemInfo.value.os_name} ${systemInfo.value.os_version}`,
      `${i18n.t("developer.kernel")}: ${systemInfo.value.kernel_version}`,
      `${i18n.t("developer.host")}: ${systemInfo.value.host_name || "-"}`,
      `${i18n.t("developer.cpu")}: ${systemInfo.value.cpu.name} (${systemInfo.value.cpu.count})`,
      `${i18n.t("developer.memory")}: ${formatBytesToGb(systemInfo.value.memory.used)} / ${formatBytesToGb(systemInfo.value.memory.total)}`,
      `${i18n.t("developer.server_instances_memory")}: ${formatBytesWithPrecision(systemInfo.value.memory.server_instances_used)}`,
      `${i18n.t("developer.app_memory")}: ${formatBytesWithPrecision(systemInfo.value.memory.app_used)}`,
      `${i18n.t("developer.process_count")}: ${systemInfo.value.process_count}`,
      `${i18n.t("developer.uptime")}: ${formatUptime(systemInfo.value.uptime)}`,
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
      if (!logLevelOptions.value.some((option) => option.value === selectedLogLevel.value)) {
        selectedLogLevel.value = ALL_LOG_LEVELS;
      }
      if (!logModuleOptions.value.some((option) => option.value === selectedLogModule.value)) {
        selectedLogModule.value = ALL_LOG_MODULES;
      }
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
      globalMessage.success(i18n.t("developer.logs_copied", { count: filteredLogCount.value }));
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

  const logPolling = useSerialPolling({
    intervalMs: LOG_POLL_INTERVAL,
    task: async () => {
      await refreshLogs();
    },
  });

  const systemPolling = useSerialPolling({
    intervalMs: SYSTEM_POLL_INTERVAL,
    task: async () => {
      await refreshSystemInfo();
    },
  });

  function startPolling() {
    if (!isEnabled.value) return;
    stopPolling();

    logPolling.start();
    systemPolling.start();
  }

  function stopPolling() {
    logPolling.stop();
    systemPolling.stop();
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
    selectedLogLevel.value = ALL_LOG_LEVELS;
    selectedLogModule.value = ALL_LOG_MODULES;
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
    filteredLogs,
    filteredLogCount,
    totalLogCount,
    hasLogEntries,
    memoryDisplayPrecision,
    selectedLogLevel,
    selectedLogModule,
    logLevelOptions,
    logModuleOptions,
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
