import { defineStore } from "pinia";
import { ref } from "vue";
import { useSettingsStore } from "./settingsStore";

const DEFAULT_MAX_LOG_LINES = 5000;

export const useConsoleStore = defineStore("console", () => {
  // Logs per server, persisted across navigation
  const logs = ref<Record<string, string[]>>({});
  // Track how many lines we have fetched per server
  const logCursors = ref<Record<string, number>>({});
  // Currently selected console server
  const activeServerId = ref<string | null>(null);

  function getMaxLogLines(): number {
    try {
      const settingsStore = useSettingsStore();
      return settingsStore.settings.max_log_lines || DEFAULT_MAX_LOG_LINES;
    } catch {
      return DEFAULT_MAX_LOG_LINES;
    }
  }

  function trimLogs(serverId: string) {
    const arr = logs.value[serverId];
    const maxLines = getMaxLogLines();
    if (arr && arr.length > maxLines) {
      logs.value[serverId] = arr.slice(-maxLines);
    }
  }

  function appendLogs(serverId: string, newLines: string[]) {
    if (!logs.value[serverId]) {
      logs.value[serverId] = [];
    }
    logs.value[serverId].push(...newLines);
    trimLogs(serverId);
  }

  function appendLocal(serverId: string, line: string) {
    if (!logs.value[serverId]) {
      logs.value[serverId] = [];
    }
    logs.value[serverId].push(line);
    trimLogs(serverId);
  }

  function getLogCursor(serverId: string): number {
    return logCursors.value[serverId] || 0;
  }

  function setLogCursor(serverId: string, cursor: number) {
    logCursors.value[serverId] = cursor;
  }

  function clearLogs(serverId: string) {
    if (logs.value[serverId]) {
      logs.value[serverId].splice(0, logs.value[serverId].length);
    } else {
      logs.value[serverId] = [];
    }
    // 不重置 cursor，避免重新读取已读过的日志
    // logCursors.value[serverId] = 0;
  }

  function setActiveServer(id: string | null) {
    activeServerId.value = id;
  }

  return {
    logs,
    logCursors,
    activeServerId,
    appendLogs,
    appendLocal,
    getLogCursor,
    setLogCursor,
    clearLogs,
    setActiveServer,
  };
});
