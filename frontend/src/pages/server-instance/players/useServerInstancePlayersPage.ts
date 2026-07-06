import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { playerApi, type ParsedPlayerLogEvent } from "@api/player";
import { serverApi, type ServerLogStructuredEvent } from "@api/server";
import { useConsoleStore } from "@stores/consoleStore";
import type { ServerInstancePlayerCardModel } from "@src/components/server-instance/ServerInstancePlayerCard.vue";
import { useNextInstanceWorkspaceContext } from "@src/composables/useNextInstanceWorkspace";

const PLAYER_LIST_SNAPSHOT_RE = /There are \d+ of a max of \d+ players online:\s*(.*)$/i;

function normalizePlayerName(name: string): string {
  return name.trim().toLowerCase();
}

function extractLatestOnlinePlayerSnapshot(logs: string[]): string[] | null {
  for (let index = logs.length - 1; index >= 0; index -= 1) {
    const match = logs[index].match(PLAYER_LIST_SNAPSHOT_RE);
    if (!match) {
      continue;
    }

    return match[1]
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }

  return null;
}

function parseOnlinePlayers(logs: string[]): string[] {
  const players: string[] = [];

  let startIndex = 0;
  for (let i = logs.length - 1; i >= 0; i -= 1) {
    const line = logs[i];
    if (/Done \([\d.]+s\)! For help/.test(line) || /Starting minecraft server/i.test(line)) {
      startIndex = i;
      break;
    }
  }

  for (let i = startIndex; i < logs.length; i += 1) {
    const line = logs[i];
    const listMatch = line.match(PLAYER_LIST_SNAPSHOT_RE);
    const joinMatch = line.match(/\]: (\w+) joined the game/);
    const loginMatch = line.match(/\]: UUID of player (\w+) is/);
    const leftMatch = line.match(/\]: (\w+) left the game/);

    if (listMatch) {
      players.splice(
        0,
        players.length,
        ...listMatch[1]
          .split(",")
          .map((item) => item.trim())
          .filter(Boolean),
      );
      continue;
    }

    if (joinMatch) {
      const name = joinMatch[1];
      if (!players.includes(name)) {
        players.push(name);
      }
    }

    if (loginMatch) {
      const name = loginMatch[1];
      if (!players.includes(name)) {
        players.push(name);
      }
    }

    if (leftMatch) {
      const name = leftMatch[1];
      const idx = players.indexOf(name);
      if (idx > -1) {
        players.splice(idx, 1);
      }
    }
  }

  return players;
}

function isStructuredPlayerEventKind(
  eventKind: ParsedPlayerLogEvent["event_kind"] | ServerLogStructuredEvent["event_kind"],
): eventKind is "server_ready" | "player_join" | "player_leave" {
  return (
    eventKind === "server_ready" || eventKind === "player_join" || eventKind === "player_leave"
  );
}

function hasStructuredPlayerEvents(events: ParsedPlayerLogEvent[]): boolean {
  return events.some((event) => isStructuredPlayerEventKind(event.event_kind));
}

function replayOnlinePlayers(events: ParsedPlayerLogEvent[]): string[] {
  const players: string[] = [];

  for (const event of events) {
    if (event.event_kind === "server_ready") {
      players.length = 0;
      continue;
    }

    if (!event.player) {
      continue;
    }

    if (event.event_kind === "player_join") {
      if (!players.includes(event.player)) {
        players.push(event.player);
      }
      continue;
    }

    if (event.event_kind === "player_leave") {
      const idx = players.indexOf(event.player);
      if (idx >= 0) {
        players.splice(idx, 1);
      }
    }
  }

  return players;
}

export function useServerInstancePlayersPage() {
  const workspace = useNextInstanceWorkspaceContext();
  const consoleStore = useConsoleStore();

  const loading = shallowRef(false);
  const refreshing = shallowRef(false);
  const errorMessage = shallowRef("");
  const structuredOnlinePlayers = shallowRef<Record<string, string[]>>({});
  const structuredPlayerEventSupport = shallowRef<Record<string, boolean>>({});
  const listSnapshotOnlinePlayers = shallowRef<Record<string, string[] | null>>({});
  const summary = shallowRef<Awaited<ReturnType<typeof playerApi.getServerPlayerSummary>> | null>(
    null,
  );
  let unlistenLogLine: UnlistenFn | null = null;
  let unlistenStructuredLog: UnlistenFn | null = null;

  const serverId = computed(() => workspace.serverId.value);
  const serverPath = computed(() => workspace.server.value?.path ?? "");
  const isRunning = computed(() => workspace.status.value?.status === "Running");

  async function ensureRecentLogsLoaded(): Promise<void> {
    if (!serverId.value) {
      return;
    }

    if ((consoleStore.logs[serverId.value] || []).length > 0) {
      return;
    }

    const lines = await serverApi.getLogs(serverId.value, 0, 5000);
    consoleStore.replaceLogs(serverId.value, lines);
    consoleStore.setLogCursor(serverId.value, lines.length);
  }

  async function pullFreshLogs(): Promise<void> {
    if (!serverId.value) {
      return;
    }

    const cursor = consoleStore.getLogCursor(serverId.value);
    const nextLines = await serverApi.getLogs(serverId.value, cursor, 5000);
    if (nextLines.length === 0) {
      return;
    }

    consoleStore.appendLogs(serverId.value, nextLines);
    consoleStore.setLogCursor(serverId.value, cursor + nextLines.length);
  }

  async function hydrateStructuredOnlinePlayers(): Promise<void> {
    if (!serverId.value) {
      return;
    }

    const logs = consoleStore.logs[serverId.value] || [];
    listSnapshotOnlinePlayers.value[serverId.value] = extractLatestOnlinePlayerSnapshot(logs);
    if (logs.length === 0) {
      structuredOnlinePlayers.value[serverId.value] = [];
      structuredPlayerEventSupport.value[serverId.value] = false;
      return;
    }

    try {
      const events = await playerApi.parsePlayerLogEvents(logs);
      structuredOnlinePlayers.value[serverId.value] = replayOnlinePlayers(events);
      structuredPlayerEventSupport.value[serverId.value] = hasStructuredPlayerEvents(events);
    } catch {
      structuredOnlinePlayers.value[serverId.value] = [];
      structuredPlayerEventSupport.value[serverId.value] = false;
    }
  }

  function resolveOnlinePlayers(): string[] {
    if (!serverId.value) {
      return [];
    }

    const snapshotPlayers = listSnapshotOnlinePlayers.value[serverId.value];
    if (snapshotPlayers) {
      return snapshotPlayers;
    }

    const structured = structuredOnlinePlayers.value[serverId.value];
    if (structuredPlayerEventSupport.value[serverId.value] && structured) {
      return structured;
    }

    return parseOnlinePlayers(consoleStore.logs[serverId.value] || []);
  }

  function ensureStructuredOnlinePlayerList(targetServerId: string): string[] {
    if (!structuredOnlinePlayers.value[targetServerId]) {
      structuredOnlinePlayers.value[targetServerId] = [];
    }
    return structuredOnlinePlayers.value[targetServerId];
  }

  function applyStructuredPlayerEvent(event: ServerLogStructuredEvent) {
    const { server_id: eventServerId, event_kind, player } = event;
    if (!isStructuredPlayerEventKind(event_kind)) {
      return;
    }

    structuredPlayerEventSupport.value[eventServerId] = true;
    listSnapshotOnlinePlayers.value[eventServerId] = null;

    if (event_kind === "server_ready") {
      structuredOnlinePlayers.value[eventServerId] = [];
      return;
    }

    if (!player) {
      return;
    }

    const players = ensureStructuredOnlinePlayerList(eventServerId);
    if (event_kind === "player_join") {
      if (!players.includes(player)) {
        players.push(player);
      }
      return;
    }

    const index = players.indexOf(player);
    if (index >= 0) {
      players.splice(index, 1);
    }
  }

  async function startSubscriptions(): Promise<void> {
    if (unlistenLogLine || !serverId.value) {
      return;
    }

    unlistenLogLine = await serverApi.onLogLine(({ server_id, line }) => {
      consoleStore.appendLocal(server_id, line);
      consoleStore.setLogCursor(server_id, (consoleStore.logs[server_id] || []).length);

      if (server_id === serverId.value) {
        const logs = consoleStore.logs[server_id] || [];
        listSnapshotOnlinePlayers.value[server_id] = extractLatestOnlinePlayerSnapshot(logs);
      }
    });

    try {
      unlistenStructuredLog = await serverApi.onStructuredLogEvent((event) => {
        applyStructuredPlayerEvent(event);
      });
    } catch {
      unlistenStructuredLog = null;
    }
  }

  function stopSubscriptions(): void {
    if (unlistenLogLine) {
      unlistenLogLine();
      unlistenLogLine = null;
    }

    if (unlistenStructuredLog) {
      unlistenStructuredLog();
      unlistenStructuredLog = null;
    }
  }

  async function loadSummary(): Promise<void> {
    if (!serverPath.value) {
      summary.value = null;
      return;
    }

    summary.value = await playerApi.getServerPlayerSummary(serverPath.value);
  }

  async function requestOnlineSnapshot(): Promise<void> {
    if (!serverId.value || !isRunning.value) {
      return;
    }

    await serverApi.sendCommand(serverId.value, "list");
    await new Promise((resolve) => window.setTimeout(resolve, 450));
    await pullFreshLogs();
    listSnapshotOnlinePlayers.value[serverId.value] = extractLatestOnlinePlayerSnapshot(
      consoleStore.logs[serverId.value] || [],
    );
  }

  async function refresh(manual = false): Promise<void> {
    if (!serverId.value) {
      return;
    }

    errorMessage.value = "";
    if (manual) {
      refreshing.value = true;
    } else {
      loading.value = true;
    }

    try {
      await workspace.refreshServerContext();
      await ensureRecentLogsLoaded();
      await pullFreshLogs();
      await loadSummary();
      await hydrateStructuredOnlinePlayers();
      if (manual) {
        await requestOnlineSnapshot();
      }
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      loading.value = false;
      refreshing.value = false;
    }
  }

  async function handleKick(name: string): Promise<void> {
    if (!serverId.value || !isRunning.value) {
      return;
    }

    await playerApi.kickPlayer(serverId.value, name);
  }

  const onlinePlayers = computed(() => resolveOnlinePlayers());
  const playerCards = computed<ServerInstancePlayerCardModel[]>(() => {
    const currentSummary = summary.value;
    const onlineNameSet = new Set(onlinePlayers.value.map((name) => normalizePlayerName(name)));
    const whitelistMap = new Map(
      (currentSummary?.whitelist ?? []).map((entry) => [normalizePlayerName(entry.name), entry]),
    );
    const bannedMap = new Map(
      (currentSummary?.banned_players ?? []).map((entry) => [
        normalizePlayerName(entry.name),
        entry,
      ]),
    );
    const opMap = new Map(
      (currentSummary?.ops ?? []).map((entry) => [normalizePlayerName(entry.name), entry]),
    );
    const allKeys = new Set<string>([
      ...onlineNameSet,
      ...Array.from(whitelistMap.keys()),
      ...Array.from(bannedMap.keys()),
      ...Array.from(opMap.keys()),
    ]);

    return Array.from(allKeys)
      .map((key) => {
        const whitelistEntry = whitelistMap.get(key);
        const bannedEntry = bannedMap.get(key);
        const opEntry = opMap.get(key);
        const name =
          onlinePlayers.value.find((item) => normalizePlayerName(item) === key) ??
          whitelistEntry?.name ??
          bannedEntry?.name ??
          opEntry?.name ??
          key;
        const uuid = whitelistEntry?.uuid ?? bannedEntry?.uuid ?? opEntry?.uuid ?? null;

        return {
          key,
          name,
          uuid,
          online: onlineNameSet.has(key),
          isOp: Boolean(opEntry),
          nameBanned: Boolean(bannedEntry),
          ipBanMatched: false,
          ipBanKnown: (currentSummary?.banned_ips.length ?? 0) > 0,
          whitelist: Boolean(whitelistEntry),
          detailBadges: [bannedEntry?.reason].filter((item): item is string => Boolean(item)),
        } satisfies ServerInstancePlayerCardModel;
      })
      .toSorted((left, right) => {
        if (left.online !== right.online) {
          return left.online ? -1 : 1;
        }
        return left.name.localeCompare(right.name);
      });
  });

  const bannedIpEntries = computed(() => summary.value?.banned_ips ?? []);

  onMounted(async () => {
    await startSubscriptions();
    await refresh(false);
  });

  onUnmounted(() => {
    stopSubscriptions();
  });

  watch(serverId, async (nextServerId, previousServerId) => {
    if (!nextServerId || nextServerId === previousServerId) {
      return;
    }

    stopSubscriptions();
    await startSubscriptions();
    await refresh(false);
  });

  return {
    loading,
    refreshing,
    errorMessage,
    isRunning,
    playerCards,
    onlineCount: computed(() => onlinePlayers.value.length),
    opCount: computed(() => summary.value?.ops.length ?? 0),
    bannedCount: computed(() => summary.value?.banned_players.length ?? 0),
    bannedIpCount: computed(() => summary.value?.banned_ips.length ?? 0),
    bannedIpEntries,
    refresh,
    handleKick,
  };
}
