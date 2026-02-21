import { defineStore } from "pinia";
import { ref } from "vue";

export const useConsoleStore = defineStore("console", () => {
  // Logs per server, persisted across navigation
  const logs = ref<Record<string, string[]>>({});
  // Track how many lines we have fetched per server
  const logCursors = ref<Record<string, number>>({});
  // Currently selected console server
  const activeServerId = ref<string | null>(null);

  function appendLogs(serverId: string, newLines: string[]) {
    if (!logs.value[serverId]) {
      logs.value[serverId] = [];
    }
    logs.value[serverId].push(...newLines);
    // Keep max 5000 lines per server
    if (logs.value[serverId].length > 5000) {
      const drain = logs.value[serverId].length - 5000;
      logs.value[serverId].splice(0, drain);
    }
  }

  function appendLocal(serverId: string, line: string) {
    if (!logs.value[serverId]) {
      logs.value[serverId] = [];
    }
    logs.value[serverId].push(line);
    if (logs.value[serverId].length > 5000) {
      const drain = logs.value[serverId].length - 5000;
      logs.value[serverId].splice(0, drain);
    }
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
