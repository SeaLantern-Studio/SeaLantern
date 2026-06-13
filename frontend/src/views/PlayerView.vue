<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useSettingsStore } from "@stores/settingsStore";
import { useRoute } from "vue-router";
import { useConsoleStore } from "@stores/consoleStore";
import { serverApi } from "@api/server";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry, type ParsedPlayerLogEvent } from "@api/player";
import { TIME, MESSAGES, getMessage } from "@utils/constants";
import { validatePlayerName, handleError } from "@utils/errorHandler";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import type { ServerLogStructuredEvent } from "@api/server";
import type { UnlistenFn } from "@tauri-apps/api/event";
import PlayerTabs from "@components/views/player/PlayerTabs.vue";
import PlayerActionBar from "@components/views/player/PlayerActionBar.vue";
import PlayerList from "@components/views/player/PlayerList.vue";
import PlayerModals from "@components/views/player/PlayerModals.vue";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const store = useServerStore();
const route = useRoute();
const consoleStore = useConsoleStore();

const activeTab = ref<PlayerTab>("online");

const whitelist = ref<PlayerEntry[]>([]);
const bannedPlayers = ref<BanEntry[]>([]);
const ops = ref<OpEntry[]>([]);

const { loading, withLoading } = useLoading();
const { error, success, showError, showSuccess, clear: clearMessage } = useMessage();

const showAddModal = ref(false);
const addPlayerName = ref("");
const addBanReason = ref("");
const addLoading = ref(false);
const settingsStore = useSettingsStore();
let unlistenLogLine: UnlistenFn | null = null;
let unlistenStructuredLog: UnlistenFn | null = null;
const structuredOnlinePlayers = ref<Record<string, string[]>>({});
const structuredPlayerEventSupport = ref<Record<string, boolean>>({});

const selectedServerId = computed(() => store.currentServerId || "");

const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === selectedServerId.value);
  return server?.path || "";
});

const isRunning = computed(() => {
  return store.statuses[selectedServerId.value]?.status === "Running";
});

const onlinePlayers = computed(() =>
  resolveOnlinePlayers(selectedServerId.value),
);

function getAddLabel(): string {
  switch (activeTab.value) {
    case "whitelist":
      return i18n.t("players.add_whitelist");
    case "banned":
      return i18n.t("players.ban_player");
    case "ops":
      return i18n.t("players.add_op");
    default:
      return i18n.t("players.add");
  }
}

onMounted(async () => {
  await settingsStore.ensureLoaded();
  await store.refreshList();
  const routeId = typeof route.params.id === "string" ? route.params.id : "";
  if (routeId && store.servers.some((server) => server.id === routeId)) {
    store.setCurrentServer(routeId);
  } else if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  await startPlayerLogSubscription();
});

onUnmounted(() => {
  stopPlayerLogSubscription();
});

watch(
  () => route.params.id,
  (routeId) => {
    const normalizedId = typeof routeId === "string" ? routeId : "";
    if (!normalizedId) {
      return;
    }
    if (store.servers.some((server) => server.id === normalizedId)) {
      store.setCurrentServer(normalizedId);
    }
  },
);

watch(
  () => store.currentServerId,
  async (serverId) => {
    if (!serverId) {
      whitelist.value = [];
      bannedPlayers.value = [];
      ops.value = [];
      structuredOnlinePlayers.value = {};
      structuredPlayerEventSupport.value = {};
      return;
    }

    await store.refreshStatus(serverId);
    await Promise.all([loadAll(), ensureRecentLogsLoaded(serverId)]);
    await hydrateStructuredOnlinePlayers(serverId);
  },
  { immediate: true },
);

async function loadAll() {
  if (!serverPath.value) return;
  await withLoading(async () => {
    whitelist.value = await playerApi.getWhitelist(serverPath.value);
    bannedPlayers.value = await playerApi.getBannedPlayers(serverPath.value);
    ops.value = await playerApi.getOps(serverPath.value);
  });
}

async function ensureRecentLogsLoaded(serverId: string) {
  if ((consoleStore.logs[serverId] || []).length > 0) {
    return;
  }

  const maxLogLines = Math.max(100, settingsStore.settings.max_log_lines || 5000);
  const lines = await serverApi.getLogs(serverId, 0, maxLogLines);
  consoleStore.replaceLogs(serverId, lines);
  consoleStore.setLogCursor(serverId, (consoleStore.logs[serverId] || []).length);
}

async function hydrateStructuredOnlinePlayers(serverId: string) {
  const logs = consoleStore.logs[serverId] || [];
  if (logs.length === 0) {
    structuredOnlinePlayers.value[serverId] = [];
    structuredPlayerEventSupport.value[serverId] = false;
    return;
  }

  try {
    const events = await playerApi.parsePlayerLogEvents(logs);
    structuredOnlinePlayers.value[serverId] = replayOnlinePlayers(events);
    structuredPlayerEventSupport.value[serverId] = hasStructuredPlayerEvents(events);
  } catch (error) {
    structuredOnlinePlayers.value[serverId] = [];
    structuredPlayerEventSupport.value[serverId] = false;
    console.warn("[PlayerView] structured player log hydration unavailable, falling back to legacy parsing", error);
  }
}

async function startPlayerLogSubscription() {
  if (unlistenLogLine) {
    return;
  }

  unlistenLogLine = await serverApi.onLogLine(({ server_id, line }) => {
    consoleStore.appendLocal(server_id, line);
    consoleStore.setLogCursor(server_id, (consoleStore.logs[server_id] || []).length);
  });

  try {
    unlistenStructuredLog = await serverApi.onStructuredLogEvent((event) => {
      applyStructuredPlayerEvent(event);
    });
  } catch (error) {
    console.warn("[PlayerView] structured log stream unavailable, falling back to legacy parsing", error);
  }
}

function stopPlayerLogSubscription() {
  if (unlistenLogLine) {
    unlistenLogLine();
    unlistenLogLine = null;
  }
  if (unlistenStructuredLog) {
    unlistenStructuredLog();
    unlistenStructuredLog = null;
  }
}

function resolveOnlinePlayers(serverId: string): string[] {
  if (!serverId) {
    return [];
  }

  const structured = structuredOnlinePlayers.value[serverId];
  if (structuredPlayerEventSupport.value[serverId] && structured) {
    return structured;
  }

  return parseOnlinePlayers(consoleStore.logs[serverId] || []);
}

function isStructuredPlayerEventKind(
  eventKind: ParsedPlayerLogEvent["event_kind"] | ServerLogStructuredEvent["event_kind"],
): eventKind is "server_ready" | "player_join" | "player_leave" {
  return eventKind === "server_ready" || eventKind === "player_join" || eventKind === "player_leave";
}

function hasStructuredPlayerEvents(events: ParsedPlayerLogEvent[]): boolean {
  return events.some((event) => isStructuredPlayerEventKind(event.event_kind));
}

function ensureStructuredOnlinePlayerList(serverId: string): string[] {
  if (!structuredOnlinePlayers.value[serverId]) {
    structuredOnlinePlayers.value[serverId] = [];
  }
  return structuredOnlinePlayers.value[serverId];
}

function applyStructuredPlayerEvent(event: ServerLogStructuredEvent) {
  const { server_id, event_kind, player } = event;
  if (!isStructuredPlayerEventKind(event_kind)) {
    return;
  }

  structuredPlayerEventSupport.value[server_id] = true;

  if (event_kind === "server_ready") {
    structuredOnlinePlayers.value[server_id] = [];
    return;
  }

  if (!player) {
    return;
  }

  const players = ensureStructuredOnlinePlayerList(server_id);
  if (event_kind === "player_join") {
    if (!players.includes(player)) {
      players.push(player);
    }
    return;
  }

  if (event_kind === "player_leave") {
    const index = players.indexOf(player);
    if (index >= 0) {
      players.splice(index, 1);
    }
  }
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
      if (idx > -1) {
        players.splice(idx, 1);
      }
    }
  }

  return players;
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
    const joinMatch = line.match(/\]: (\w+) joined the game/);
    const loginMatch = line.match(/\]: UUID of player (\w+) is/);
    const leftMatch = line.match(/\]: (\w+) left the game/);

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

function schedulePlayerListRefresh() {
  window.setTimeout(() => {
    void loadAll();
  }, TIME.SUCCESS_MESSAGE_DURATION);
}

function openAddModal() {
  addPlayerName.value = "";
  addBanReason.value = "";
  showAddModal.value = true;
}

async function handleAdd() {
  const validation = validatePlayerName(addPlayerName.value);
  if (!validation.valid) {
    showError(validation.error || getMessage(MESSAGES.ERROR.INVALID_PLAYER_NAME));
    return;
  }

  if (!isRunning.value) {
    showError(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }

  addLoading.value = true;
  try {
    const sid = selectedServerId.value;
    switch (activeTab.value) {
      case "whitelist":
        await playerApi.addToWhitelist(sid, addPlayerName.value);
        showSuccess(getMessage(MESSAGES.SUCCESS.WHITELIST_ADDED));
        break;
      case "banned":
        await playerApi.banPlayer(sid, addPlayerName.value, addBanReason.value);
        showSuccess(getMessage(MESSAGES.SUCCESS.PLAYER_BANNED));
        break;
      case "ops":
        await playerApi.addOp(sid, addPlayerName.value);
        showSuccess(getMessage(MESSAGES.SUCCESS.OP_ADDED));
        break;
    }
    showAddModal.value = false;
    schedulePlayerListRefresh();
  } catch (e) {
    showError(handleError(e, "AddPlayer"));
  } finally {
    addLoading.value = false;
  }
}

async function handleRemoveWhitelist(name: string) {
  if (!isRunning.value) {
    showError(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.removeFromWhitelist(selectedServerId.value, name);
    showSuccess(getMessage(MESSAGES.SUCCESS.WHITELIST_REMOVED));
    schedulePlayerListRefresh();
  } catch (e) {
    showError(handleError(e, "RemoveWhitelist"));
  }
}

async function handleUnban(name: string) {
  if (!isRunning.value) {
    showError(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.unbanPlayer(selectedServerId.value, name);
    showSuccess(getMessage(MESSAGES.SUCCESS.PLAYER_UNBANNED));
    schedulePlayerListRefresh();
  } catch (e) {
    showError(handleError(e, "UnbanPlayer"));
  }
}

async function handleRemoveOp(name: string) {
  if (!isRunning.value) {
    showError(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.removeOp(selectedServerId.value, name);
    showSuccess(getMessage(MESSAGES.SUCCESS.OP_REMOVED));
    schedulePlayerListRefresh();
  } catch (e) {
    showError(handleError(e, "RemoveOp"));
  }
}

async function handleKick(name: string) {
  if (!isRunning.value) {
    showError(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.kickPlayer(selectedServerId.value, name);
    showSuccess(`${name} ${getMessage(MESSAGES.SUCCESS.PLAYER_KICKED)}`);
  } catch (e) {
    showError(handleError(e, "KickPlayer"));
  }
}
</script>

<template>
  <div class="player-view animate-fade-in-up">
    <div v-if="!selectedServerId" class="player-empty-state">
      <p class="text-body">{{ i18n.t("players.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="player-msg-banner error-banner">
        <span>{{ error }}</span>
        <button @click="clearMessage('error')">x</button>
      </div>
      <div v-if="success" class="player-msg-banner success-banner">
        <span>{{ success }}</span>
      </div>

      <PlayerTabs
        v-model="activeTab"
        :onlineCount="onlinePlayers.length"
        :whitelistCount="whitelist.length"
        :bannedCount="bannedPlayers.length"
        :opsCount="ops.length"
      />

      <PlayerActionBar
        v-if="activeTab !== 'online'"
        :label="getAddLabel()"
        :disabled="!isRunning"
        @add="openAddModal"
        @refresh="loadAll"
      />

      <PlayerList
        :loading="loading"
        :tab="activeTab"
        :onlinePlayers="onlinePlayers"
        :whitelist="whitelist"
        :bannedPlayers="bannedPlayers"
        :ops="ops"
        :serverRunning="isRunning"
        @kick="handleKick"
        @removeWhitelist="handleRemoveWhitelist"
        @unban="handleUnban"
        @removeOp="handleRemoveOp"
      />

      <PlayerModals
        v-model:visible="showAddModal"
        :title="getAddLabel()"
        :showBanReason="activeTab === 'banned'"
        :loading="addLoading"
        :serverRunning="isRunning"
        v-model:playerName="addPlayerName"
        v-model:banReason="addBanReason"
        @confirm="handleAdd"
      />
    </template>
  </div>
</template>

<style scoped>
.player-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.player-empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
}

.player-msg-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}

.error-banner {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--sl-error);
}

.success-banner {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  color: var(--sl-success);
}

.player-msg-banner button {
  font-weight: 600;
  color: inherit;
}
</style>
