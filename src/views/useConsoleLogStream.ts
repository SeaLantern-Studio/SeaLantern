import { nextTick, onUnmounted, type Ref } from "vue";
import { serverApi } from "@api/server";
import { useConsoleStore } from "@stores/consoleStore";
import type { UnlistenFn } from "@tauri-apps/api/event";

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: () => void;
  getAllPlainText: () => string;
}

interface UseConsoleLogStreamOptions {
  consoleOutputRef: Ref<ConsoleOutputExpose | null>;
  userScrolledUp: Ref<boolean>;
  maxLogLines: Ref<number>;
  doScroll: () => void;
}

export function useConsoleLogStream(options: UseConsoleLogStreamOptions) {
  const consoleStore = useConsoleStore();
  let unlistenLogLine: UnlistenFn | null = null;

  async function startLogSubscription(serverId: string) {
    if (unlistenLogLine) {
      return;
    }

    unlistenLogLine = await serverApi.onLogLine(({ server_id, line }) => {
      consoleStore.appendLocal(server_id, line);
      consoleStore.setLogCursor(server_id, (consoleStore.logs[server_id] || []).length);
      if (!serverId || server_id !== serverId) return;
      options.consoleOutputRef.value?.appendLines([line]);
    });
  }

  function stopLogSubscription() {
    if (!unlistenLogLine) {
      return;
    }

    unlistenLogLine();
    unlistenLogLine = null;
  }

  function renderCachedLogs(serverId: string) {
    options.consoleOutputRef.value?.clear();
    const lines = consoleStore.logs[serverId] || [];
    if (lines.length > 0) {
      options.consoleOutputRef.value?.appendLines(lines);
    }
  }

  async function syncLogsOnce(serverId: string) {
    try {
      const cachedLines = consoleStore.logs[serverId] || [];

      if (cachedLines.length > 0) {
        renderCachedLogs(serverId);
        return;
      }

      const lines = await serverApi.getLogs(serverId, 0, Math.max(1, options.maxLogLines.value));
      consoleStore.replaceLogs(serverId, lines);
      consoleStore.setLogCursor(serverId, (consoleStore.logs[serverId] || []).length);
      renderCachedLogs(serverId);
    } catch (_error) {}
  }

  async function activateLogStream(serverId: string) {
    if (!serverId) {
      return;
    }

    consoleStore.setActiveServer(serverId);
    renderCachedLogs(serverId);
    await startLogSubscription(serverId);
  }

  function deactivateLogStream() {
    stopLogSubscription();
    consoleStore.setActiveServer(null);
  }

  async function syncAndFocus(serverId: string) {
    if (!serverId) {
      return;
    }

    await syncLogsOnce(serverId);
    options.userScrolledUp.value = false;
    nextTick(() => options.doScroll());
  }

  function clearActiveLogs(serverId: string) {
    if (serverId) {
      consoleStore.clearLogs(serverId);
      consoleStore.setLogCursor(serverId, 0);
    }

    options.consoleOutputRef.value?.clear();
  }

  function appendCommandEcho(serverId: string, command: string) {
    consoleStore.appendLocal(serverId, "> " + command);
    consoleStore.setLogCursor(serverId, (consoleStore.logs[serverId] || []).length);
    options.consoleOutputRef.value?.appendLines(["> " + command]);
  }

  onUnmounted(() => {
    stopLogSubscription();
  });

  return {
    startLogSubscription,
    stopLogSubscription,
    renderCachedLogs,
    syncLogsOnce,
    activateLogStream,
    deactivateLogStream,
    syncAndFocus,
    clearActiveLogs,
    appendCommandEcho,
  };
}
