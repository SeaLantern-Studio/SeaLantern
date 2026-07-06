import { defineStore } from "pinia";
import { ref } from "vue";

const CONSOLE_HISTORY_STORAGE_KEY = "sea-lantern.console-history";
const MAX_HISTORY_PER_SERVER = 100;

type ConsoleHistoryByServer = Record<string, string[]>;

function sanitizeHistoryPayload(value: unknown): ConsoleHistoryByServer {
  if (typeof value !== "object" || value === null) {
    return {};
  }

  const result: ConsoleHistoryByServer = {};

  for (const [serverId, commands] of Object.entries(value)) {
    if (!Array.isArray(commands)) {
      continue;
    }

    const sanitized = commands
      .filter((command): command is string => typeof command === "string")
      .map((command) => command.trim())
      .filter((command) => command.length > 0)
      .slice(-MAX_HISTORY_PER_SERVER);

    if (sanitized.length > 0) {
      result[serverId] = sanitized;
    }
  }

  return result;
}

export const useConsoleHistoryStore = defineStore("consoleHistory", () => {
  const historyByServer = ref<ConsoleHistoryByServer>({});
  let loaded = false;

  function loadHistory(): void {
    if (loaded || typeof window === "undefined") {
      return;
    }

    loaded = true;

    try {
      const raw = window.localStorage.getItem(CONSOLE_HISTORY_STORAGE_KEY);
      if (!raw) {
        return;
      }

      historyByServer.value = sanitizeHistoryPayload(JSON.parse(raw));
    } catch {
      historyByServer.value = {};
    }
  }

  function persistHistory(): void {
    if (typeof window === "undefined") {
      return;
    }

    try {
      window.localStorage.setItem(
        CONSOLE_HISTORY_STORAGE_KEY,
        JSON.stringify(historyByServer.value),
      );
    } catch {
      // Ignore persistence failures and keep in-memory history available.
    }
  }

  function getHistory(serverId: string | null | undefined): string[] {
    loadHistory();
    if (!serverId) {
      return [];
    }

    return historyByServer.value[serverId] ?? [];
  }

  function pushCommand(serverId: string | null | undefined, command: string): void {
    loadHistory();

    if (!serverId) {
      return;
    }

    const normalized = command.trim();
    if (!normalized) {
      return;
    }

    const existing = historyByServer.value[serverId] ?? [];
    historyByServer.value[serverId] = [...existing, normalized].slice(-MAX_HISTORY_PER_SERVER);
    persistHistory();
  }

  return {
    historyByServer,
    getHistory,
    pushCommand,
  };
});
