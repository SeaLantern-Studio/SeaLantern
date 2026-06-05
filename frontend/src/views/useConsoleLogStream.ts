import { nextTick, onUnmounted, type Ref } from "vue";
import { serverApi } from "@api/server";
import { useSerialPolling } from "@composables/useSerialPolling";
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

function arraysEqual(left: string[], right: string[]): boolean {
  if (left.length !== right.length) {
    return false;
  }

  return left.every((line, index) => line === right[index]);
}

function isPrefixOf(full: string[], prefix: string[]): boolean {
  if (prefix.length > full.length) {
    return false;
  }

  return prefix.every((line, index) => full[index] === line);
}

function findOverlapSuffix(existing: string[], next: string[]): number {
  const maxOverlap = Math.min(existing.length, next.length);
  for (let size = maxOverlap; size > 0; size -= 1) {
    let matches = true;
    for (let index = 0; index < size; index += 1) {
      if (existing[existing.length - size + index] !== next[index]) {
        matches = false;
        break;
      }
    }

    if (matches) {
      return size;
    }
  }

  return 0;
}

export function useConsoleLogStream(options: UseConsoleLogStreamOptions) {
  const consoleStore = useConsoleStore();
  const LOG_SYNC_INTERVAL_MS = 1200;
  const SYSTEM_LOG_PREFIX = "[Sea Lantern]";
  const RECENT_SYSTEM_LOG_DEDUP_MS = 2500;
  let unlistenLogLine: UnlistenFn | null = null;
  let activeServerId: string | null = null;
  const recentSystemLogByServer = new Map<string, { line: string; at: number }>();

  function filterRecentSystemLogDuplicates(serverId: string, lines: string[]): string[] {
    const now = Date.now();
    const filtered: string[] = [];

    for (const line of lines) {
      if (!line.startsWith(SYSTEM_LOG_PREFIX)) {
        filtered.push(line);
        continue;
      }

      const recent = recentSystemLogByServer.get(serverId);
      const isDuplicate =
        recent !== undefined &&
        recent.line === line &&
        now - recent.at <= RECENT_SYSTEM_LOG_DEDUP_MS;

      if (isDuplicate) {
        continue;
      }

      recentSystemLogByServer.set(serverId, { line, at: now });
      filtered.push(line);
    }

    return filtered;
  }

  function syncRecentSystemLogMarker(serverId: string, lines: string[]) {
    for (let index = lines.length - 1; index >= 0; index -= 1) {
      const line = lines[index];
      if (!line.startsWith(SYSTEM_LOG_PREFIX)) {
        continue;
      }

      recentSystemLogByServer.set(serverId, { line, at: Date.now() });
      return;
    }

    recentSystemLogByServer.delete(serverId);
  }

  async function syncRecentLogs(serverId: string) {
    const lines = await serverApi.getLogs(serverId, 0, Math.max(1, options.maxLogLines.value));
    const cachedLines = consoleStore.logs[serverId] || [];

    if (arraysEqual(cachedLines, lines)) {
      return;
    }

    if (cachedLines.length > lines.length && isPrefixOf(cachedLines, lines)) {
      return;
    }

    const overlap = findOverlapSuffix(cachedLines, lines);
    consoleStore.replaceLogs(serverId, lines);
    consoleStore.setLogCursor(serverId, lines.length);

    if (activeServerId !== serverId) {
      return;
    }

    if (cachedLines.length > 0 && overlap > 0 && overlap < lines.length) {
      const appendedLines = filterRecentSystemLogDuplicates(serverId, lines.slice(overlap));
      if (appendedLines.length > 0) {
        options.consoleOutputRef.value?.appendLines(appendedLines);
      }
      return;
    }

    renderCachedLogs(serverId);
  }

  const logSyncPolling = useSerialPolling({
    intervalMs: LOG_SYNC_INTERVAL_MS,
    task: async () => {
      if (!activeServerId) {
        return;
      }

      await syncRecentLogs(activeServerId);
    },
  });

  async function startLogSubscription(serverId: string) {
    if (unlistenLogLine) {
      return;
    }

    unlistenLogLine = await serverApi.onLogLine(({ server_id, line }) => {
      const dedupedLines = filterRecentSystemLogDuplicates(server_id, [line]);
      if (dedupedLines.length === 0) {
        return;
      }

      consoleStore.appendLocal(server_id, dedupedLines[0]);
      consoleStore.setLogCursor(server_id, (consoleStore.logs[server_id] || []).length);
      if (!serverId || server_id !== serverId) return;
      options.consoleOutputRef.value?.appendLines(dedupedLines);
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
    syncRecentSystemLogMarker(serverId, lines);
  }

  async function syncLogsOnce(serverId: string) {
    try {
      await syncRecentLogs(serverId);
    } catch (_error) {}
  }

  async function activateLogStream(serverId: string) {
    if (!serverId) {
      return;
    }

    activeServerId = serverId;
    consoleStore.setActiveServer(serverId);
    renderCachedLogs(serverId);
    await syncRecentLogs(serverId);
    await startLogSubscription(serverId);
    logSyncPolling.start();
  }

  function deactivateLogStream() {
    if (activeServerId) {
      recentSystemLogByServer.delete(activeServerId);
    }
    activeServerId = null;
    logSyncPolling.stop();
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
      recentSystemLogByServer.delete(serverId);
    }

    options.consoleOutputRef.value?.clear();
  }

  function appendCommandEcho(serverId: string, command: string) {
    consoleStore.appendLocal(serverId, "> " + command);
    options.consoleOutputRef.value?.appendLines(["> " + command]);
  }

  onUnmounted(() => {
    logSyncPolling.stop();
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
