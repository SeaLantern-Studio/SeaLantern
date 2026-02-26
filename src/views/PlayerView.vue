<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useConsoleStore } from "@stores/consoleStore";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry } from "@api/player";
import { TIME, MESSAGES } from "@utils/constants";
import { validatePlayerName, handleError } from "@utils/errorHandler";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import PlayerTabs from "@components/views/player/PlayerTabs.vue";
import PlayerActionBar from "@components/views/player/PlayerActionBar.vue";
import PlayerList from "@components/views/player/PlayerList.vue";
import PlayerModals from "@components/views/player/PlayerModals.vue";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const store = useServerStore();
const consoleStore = useConsoleStore();

const activeTab = ref<PlayerTab>("online");

const whitelist = ref<PlayerEntry[]>([]);
const bannedPlayers = ref<BanEntry[]>([]);
const ops = ref<OpEntry[]>([]);
const onlinePlayers = ref<string[]>([]);

const { loading, withLoading } = useLoading();
const { error, success, showError, showSuccess, clear: clearMessage } = useMessage();

const showAddModal = ref(false);
const addPlayerName = ref("");
const addBanReason = ref("");
const addLoading = ref(false);

let refreshTimer: ReturnType<typeof setInterval> | null = null;

const selectedServerId = computed(() => store.currentServerId || "");

const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === selectedServerId.value);
  return server?.path || "";
});

const isRunning = computed(() => {
  return store.statuses[selectedServerId.value]?.status === "Running";
});

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
  await store.refreshList();
  if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  if (store.currentServerId) {
    await store.refreshStatus(store.currentServerId);
    await loadAll();
    parseOnlinePlayers();
  }
  startRefresh();
});

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer);
});

function startRefresh() {
  if (refreshTimer) clearInterval(refreshTimer);
  refreshTimer = setInterval(async () => {
    if (selectedServerId.value) {
      await store.refreshStatus(selectedServerId.value);
      await loadAll();
      parseOnlinePlayers();
    }
  }, 5000);
}

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      await store.refreshStatus(store.currentServerId);
      await loadAll();
      parseOnlinePlayers();
    }
  },
);

async function loadAll() {
  if (!serverPath.value) return;
  await withLoading(async () => {
    whitelist.value = await playerApi.getWhitelist(serverPath.value);
    bannedPlayers.value = await playerApi.getBannedPlayers(serverPath.value);
    ops.value = await playerApi.getOps(serverPath.value);
  });
}

function parseOnlinePlayers() {
  const sid = selectedServerId.value;
  const logs = consoleStore.logs[sid] || [];
  const players: string[] = [];

  let startIndex = 0;
  for (let i = logs.length - 1; i >= 0; i--) {
    const line = logs[i];
    if (/Done \([\d.]+s\)! For help/.test(line) || /Starting minecraft server/i.test(line)) {
      startIndex = i;
      break;
    }
  }

  for (let i = startIndex; i < logs.length; i++) {
    const line = logs[i];
    const joinMatch = line.match(/\]: (\w+) joined the game/);
    const loginMatch = line.match(/\]: UUID of player (\w+) is/);
    const leftMatch = line.match(/\]: (\w+) left the game/);

    if (joinMatch) {
      const name = joinMatch[1];
      if (!players.includes(name)) players.push(name);
    }
    if (loginMatch) {
      const name = loginMatch[1];
      if (!players.includes(name)) players.push(name);
    }
    if (leftMatch) {
      const name = leftMatch[1];
      const idx = players.indexOf(name);
      if (idx > -1) players.splice(idx, 1);
    }
  }

  onlinePlayers.value = players;
}

function openAddModal() {
  addPlayerName.value = "";
  addBanReason.value = "";
  showAddModal.value = true;
}

async function handleAdd() {
  const validation = validatePlayerName(addPlayerName.value);
  if (!validation.valid) {
    showError(validation.error || MESSAGES.ERROR.INVALID_PLAYER_NAME);
    return;
  }

  if (!isRunning.value) {
    showError(MESSAGES.ERROR.SERVER_NOT_RUNNING);
    return;
  }

  addLoading.value = true;
  try {
    const sid = selectedServerId.value;
    switch (activeTab.value) {
      case "whitelist":
        await playerApi.addToWhitelist(sid, addPlayerName.value);
        showSuccess(MESSAGES.SUCCESS.WHITELIST_ADDED);
        break;
      case "banned":
        await playerApi.banPlayer(sid, addPlayerName.value, addBanReason.value);
        showSuccess(MESSAGES.SUCCESS.PLAYER_BANNED);
        break;
      case "ops":
        await playerApi.addOp(sid, addPlayerName.value);
        showSuccess(MESSAGES.SUCCESS.OP_ADDED);
        break;
    }
    showAddModal.value = false;
    setTimeout(() => {
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    showError(handleError(e, "AddPlayer"));
  } finally {
    addLoading.value = false;
  }
}

async function handleRemoveWhitelist(name: string) {
  if (!isRunning.value) {
    showError(MESSAGES.ERROR.SERVER_NOT_RUNNING);
    return;
  }
  try {
    await playerApi.removeFromWhitelist(selectedServerId.value, name);
    showSuccess(MESSAGES.SUCCESS.WHITELIST_REMOVED);
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    showError(handleError(e, "RemoveWhitelist"));
  }
}

async function handleUnban(name: string) {
  if (!isRunning.value) {
    showError(MESSAGES.ERROR.SERVER_NOT_RUNNING);
    return;
  }
  try {
    await playerApi.unbanPlayer(selectedServerId.value, name);
    showSuccess(MESSAGES.SUCCESS.PLAYER_UNBANNED);
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    showError(handleError(e, "UnbanPlayer"));
  }
}

async function handleRemoveOp(name: string) {
  if (!isRunning.value) {
    showError(MESSAGES.ERROR.SERVER_NOT_RUNNING);
    return;
  }
  try {
    await playerApi.removeOp(selectedServerId.value, name);
    showSuccess(MESSAGES.SUCCESS.OP_REMOVED);
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    showError(handleError(e, "RemoveOp"));
  }
}

async function handleKick(name: string) {
  if (!isRunning.value) {
    showError(MESSAGES.ERROR.SERVER_NOT_RUNNING);
    return;
  }
  try {
    await playerApi.kickPlayer(selectedServerId.value, name);
    showSuccess(`${name} ${MESSAGES.SUCCESS.PLAYER_KICKED}`);
    setTimeout(() => parseOnlinePlayers(), TIME.SUCCESS_MESSAGE_DURATION);
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
