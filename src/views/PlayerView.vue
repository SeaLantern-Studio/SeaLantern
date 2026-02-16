<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
import { useRoute } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLModal from "../components/common/SLModal.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry } from "../api/player";
import { serverApi } from "../api/server";
import { TIME, MESSAGES } from "../utils/constants";
import { validatePlayerName, handleError } from "../utils/errorHandler";
import { i18n } from "../locales";

const route = useRoute();
const store = useServerStore();
const consoleStore = useConsoleStore();

const activeTab = ref<"online" | "whitelist" | "banned" | "ops">('online');
const tabIndicator = ref<HTMLElement | null>(null);

const whitelist = ref<PlayerEntry[]>([]);
const bannedPlayers = ref<BanEntry[]>([]);
const ops = ref<OpEntry[]>([]);
const onlinePlayers = ref<string[]>([]);

const loading = ref(false);
const error = ref<string | null>(null);
const success = ref<string | null>(null);

const showAddModal = ref(false);
const addPlayerName = ref("");
const addBanReason = ref("");
const addLoading = ref(false);

let refreshTimer: ReturnType<typeof setInterval> | null = null;

const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === store.currentServerId);
  return server?.path || "";
});

const isRunning = computed(() => {
  return store.statuses[store.currentServerId]?.status === "Running";
});

const currentServerId = computed(() => store.currentServerId);

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) store.setCurrentServer(routeId);
  else if (!store.currentServerId && store.servers.length > 0)
    store.setCurrentServer(store.servers[0].id);

  startRefresh();
  if (store.currentServerId) {
    await store.refreshStatus(store.currentServerId);
    await loadAll();
    parseOnlinePlayers();
  }
});

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer);
});

function startRefresh() {
  if (refreshTimer) clearInterval(refreshTimer);
  refreshTimer = setInterval(async () => {
    if (store.currentServerId) {
      await store.refreshStatus(store.currentServerId);
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
  loading.value = true;
  error.value = null;
  try {
    whitelist.value = await playerApi.getWhitelist(serverPath.value);
    bannedPlayers.value = await playerApi.getBannedPlayers(serverPath.value);
    ops.value = await playerApi.getOps(serverPath.value);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function parseOnlinePlayers() {
  const sid = store.currentServerId;
  const logs = consoleStore.logs[sid] || [];
  const players = new Set<string>();

  for (const line of logs) {
    const joinMatch = line.match(/\]: (\w+) joined the game/);
    const leftMatch = line.match(/\]: (\w+) left the game/);
    const lostConnectionMatch = line.match(/\]: (\w+)(?: \([^)]+\))? lost connection:/);
    const disconnectingMatch = line.match(/\]: Disconnecting (\w+)(?: \([^)]+\))?:/);

    if (joinMatch) {
      const name = joinMatch[1];
      players.add(name);
    }
    if (leftMatch) {
      const name = leftMatch[1];
      players.delete(name);
    }
    if (lostConnectionMatch) {
      const name = lostConnectionMatch[1];
      players.delete(name);
    }
    if (disconnectingMatch) {
      const name = disconnectingMatch[1];
      players.delete(name);
    }
  }

  onlinePlayers.value = Array.from(players);
}

function openAddModal() {
  addPlayerName.value = "";
  addBanReason.value = "";
  showAddModal.value = true;
}

async function handleAdd() {
  // 验证玩家名
  const validation = validatePlayerName(addPlayerName.value);
  if (!validation.valid) {
    error.value = validation.error || MESSAGES.ERROR.INVALID_PLAYER_NAME;
    return;
  }

  if (!isRunning.value) {
    error.value = MESSAGES.ERROR.SERVER_NOT_RUNNING;
    return;
  }

  addLoading.value = true;
  error.value = null;
  try {
    const sid = store.currentServerId;
    switch (activeTab.value) {
      case "whitelist":
        await playerApi.addToWhitelist(sid, addPlayerName.value);
        success.value = MESSAGES.SUCCESS.WHITELIST_ADDED;
        break;
      case "banned":
        await playerApi.banPlayer(sid, addPlayerName.value, addBanReason.value);
        success.value = MESSAGES.SUCCESS.PLAYER_BANNED;
        break;
      case "ops":
        await playerApi.addOp(sid, addPlayerName.value);
        success.value = MESSAGES.SUCCESS.OP_ADDED;
        break;
    }
    showAddModal.value = false;
    setTimeout(() => {
      success.value = null;
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    error.value = handleError(e, "AddPlayer");
  } finally {
    addLoading.value = false;
  }
}

async function handleRemoveWhitelist(name: string) {
  if (!isRunning.value) {
    error.value = MESSAGES.ERROR.SERVER_NOT_RUNNING;
    return;
  }
  try {
    await playerApi.removeFromWhitelist(store.currentServerId, name);
    success.value = MESSAGES.SUCCESS.WHITELIST_REMOVED;
    setTimeout(() => {
      success.value = null;
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    error.value = handleError(e, "RemoveWhitelist");
  }
}

async function handleUnban(name: string) {
  if (!isRunning.value) {
    error.value = MESSAGES.ERROR.SERVER_NOT_RUNNING;
    return;
  }
  try {
    await playerApi.unbanPlayer(store.currentServerId, name);
    success.value = MESSAGES.SUCCESS.PLAYER_UNBANNED;
    setTimeout(() => {
      success.value = null;
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    error.value = handleError(e, "UnbanPlayer");
  }
}

async function handleRemoveOp(name: string) {
  if (!isRunning.value) {
    error.value = MESSAGES.ERROR.SERVER_NOT_RUNNING;
    return;
  }
  try {
    await playerApi.removeOp(store.currentServerId, name);
    success.value = MESSAGES.SUCCESS.OP_REMOVED;
    setTimeout(() => {
      success.value = null;
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    error.value = handleError(e, "RemoveOp");
  }
}

async function handleKick(name: string) {
  if (!isRunning.value) {
    error.value = MESSAGES.ERROR.SERVER_NOT_RUNNING;
    return;
  }
  try {
    await playerApi.kickPlayer(store.currentServerId, name);
    success.value = `${name} ${MESSAGES.SUCCESS.PLAYER_KICKED}`;
    setTimeout(() => {
      success.value = null;
      parseOnlinePlayers();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    error.value = handleError(e, "KickPlayer");
  }
}

function getAddLabel(): string {
  switch (activeTab.value) {
    case "whitelist":
      return i18n.t("players.add");
    case "banned":
      return i18n.t("players.ban_player");
    case "ops":
      return i18n.t("players.add_op");
    default:
      return i18n.t("players.add");
  }
}

// 选择标签并更新指示器位置
function selectTab(tab: "online" | "whitelist" | "banned" | "ops") {
  activeTab.value = tab;
  updateTabIndicator();
}

// 更新标签指示器位置
function updateTabIndicator() {
  setTimeout(() => {
    if (!tabIndicator.value) return;
    
    const activeTabBtn = document.querySelector('.tab-btn.active');
    if (activeTabBtn) {
      const { offsetLeft, offsetWidth } = activeTabBtn as HTMLElement;
      tabIndicator.value.style.left = `${offsetLeft}px`;
      tabIndicator.value.style.width = `${offsetWidth}px`;
    }
  }, 100); // 添加延迟，确保DOM已完全渲染
}

// 监听标签变化，更新指示器位置
watch(activeTab, () => {
  updateTabIndicator();
});

// 组件挂载后初始化指示器位置
onMounted(() => {
  // 原有代码...
  updateTabIndicator();
});
</script>

<template>
  <div class="player-view animate-fade-in-up">
    <div class="player-header">
      <div v-if="currentServerId" class="server-status">
        <SLBadge
          :text="isRunning ? i18n.t('home.running') : i18n.t('home.stopped')"
          :variant="isRunning ? 'success' : 'neutral'"
        />
        <span v-if="!isRunning" class="status-hint text-caption">{{
          i18n.t("players.server_not_running")
        }}</span>
      </div>
    </div>

    <div v-if="!currentServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("players.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="msg-banner error-banner">
        <span>{{ error }}</span>
        <button @click="error = null">x</button>
      </div>
      <div v-if="success" class="msg-banner success-banner">
        <span>{{ success }}</span>
      </div>

      <div class="tab-bar">
        <div class="tab-indicator" ref="tabIndicator"></div>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'online' }"
          @click="selectTab('online')"
        >
          {{ i18n.t("players.online_players") }}
          <span class="tab-count">{{ onlinePlayers.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'whitelist' }"
          @click="selectTab('whitelist')"
        >
          {{ i18n.t("players.whitelist") }} <span class="tab-count">{{ whitelist.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'banned' }"
          @click="selectTab('banned')"
        >
          {{ i18n.t("players.banned") }} <span class="tab-count">{{ bannedPlayers.length }}</span>
        </button>
        <button class="tab-btn" :class="{ active: activeTab === 'ops' }" @click="selectTab('ops')">
          {{ i18n.t("players.ops") }} <span class="tab-count">{{ ops.length }}</span>
        </button>
      </div>

      <div v-if="activeTab !== 'online'" class="action-bar">
        <SLButton variant="primary" size="sm" :disabled="!isRunning" @click="openAddModal">{{
          getAddLabel()
        }}</SLButton>
        <SLButton variant="ghost" size="sm" @click="loadAll">{{
          i18n.t("config.reload")
        }}</SLButton>
      </div>

      <div v-if="loading" class="loading-state">
        <div class="spinner"></div>
        <span>{{ i18n.t("config.loading") }}</span>
      </div>

      <!-- Online Tab -->
      <div v-else-if="activeTab === 'online'" class="player-list">
        <div v-if="!isRunning" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.server_offline") }}</p>
        </div>
        <div v-else-if="onlinePlayers.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.no_players") }}</p>
        </div>
        <div v-for="name in onlinePlayers" :key="name" class="player-item glass-card">
          <div class="player-avatar">
            <img
              :src="'https://mc-heads.net/avatar/' + name + '/32'"
              :alt="name"
              class="avatar-img"
            />
          </div>
          <div class="player-info">
            <span class="player-name">{{ name }}</span>
            <SLBadge :text="i18n.t('home.running')" variant="success" />
          </div>
          <div class="player-actions">
            <SLButton variant="ghost" size="sm" @click="handleKick(name)">{{
              i18n.t("players.kick")
            }}</SLButton>
          </div>
        </div>
      </div>

      <!-- Whitelist Tab -->
      <div v-else-if="activeTab === 'whitelist'" class="player-list">
        <div v-if="whitelist.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_whitelist") }}</p>
        </div>
        <div v-for="p in whitelist" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img :src="'https://mc-heads.net/avatar/' + p.name + '/32'" class="avatar-img" />
          </div>
          <div class="player-info">
            <span class="player-name">{{ p.name }}</span>
            <span class="player-uuid text-mono text-caption">{{ p.uuid }}</span>
          </div>
          <div class="player-actions">
            <SLButton
              variant="ghost"
              size="sm"
              :disabled="!isRunning"
              @click="handleRemoveWhitelist(p.name)"
              >{{ i18n.t("players.remove") }}</SLButton
            >
          </div>
        </div>
      </div>

      <!-- Banned Tab -->
      <div v-else-if="activeTab === 'banned'" class="player-list">
        <div v-if="bannedPlayers.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_banned") }}</p>
        </div>
        <div v-for="p in bannedPlayers" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img :src="'https://mc-heads.net/avatar/' + p.name + '/32'" class="avatar-img" />
          </div>
          <div class="player-info">
            <span class="player-name">{{ p.name }}</span>
            <span class="text-caption"
              >{{ i18n.t("players.ban_reason") }}: {{ p.reason || i18n.t("players.empty") }}</span
            >
          </div>
          <SLBadge :text="i18n.t('players.ban')" variant="error" />
          <div class="player-actions">
            <SLButton
              variant="ghost"
              size="sm"
              :disabled="!isRunning"
              @click="handleUnban(p.name)"
              >{{ i18n.t("players.unban") }}</SLButton
            >
          </div>
        </div>
      </div>

      <!-- OPs Tab -->
      <div v-else-if="activeTab === 'ops'" class="player-list">
        <div v-if="ops.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_ops") }}</p>
        </div>
        <div v-for="p in ops" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img :src="'https://mc-heads.net/avatar/' + p.name + '/32'" class="avatar-img" />
          </div>
          <div class="player-info">
            <span class="player-name">{{ p.name }}</span>
            <span class="text-caption">{{ i18n.t("players.level") }}: {{ p.level }}</span>
          </div>
          <SLBadge :text="i18n.t('players.ops')" variant="warning" />
          <div class="player-actions">
            <SLButton
              variant="ghost"
              size="sm"
              :disabled="!isRunning"
              @click="handleRemoveOp(p.name)"
              >{{ i18n.t("players.deop") }}</SLButton
            >
          </div>
        </div>
      </div>
    </template>

    <SLModal :visible="showAddModal" :title="getAddLabel()" @close="showAddModal = false">
      <div class="modal-form">
        <SLInput
          :label="i18n.t('players.player_name')"
          :placeholder="i18n.t('players.player_id')"
          v-model="addPlayerName"
        />
        <SLInput
          v-if="activeTab === 'banned'"
          :label="i18n.t('players.ban_reason')"
          :placeholder="i18n.t('players.ban_reason_placeholder')"
          v-model="addBanReason"
        />
        <p v-if="!isRunning" class="text-error" style="font-size: 0.8125rem">
          {{ i18n.t("players.server_not_running_hint") }}
        </p>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showAddModal = false">{{
          i18n.t("players.cancel")
        }}</SLButton>
        <SLButton
          variant="primary"
          :loading="addLoading"
          :disabled="!isRunning"
          @click="handleAdd"
          >{{ i18n.t("players.confirm") }}</SLButton
        >
      </template>
    </SLModal>
  </div>
</template>

<style scoped>
.player-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.player-header {
  display: flex;
  align-items: flex-end;
  gap: var(--sl-space-lg);
}
.server-picker {
  min-width: 300px;
}
.server-status {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding-bottom: 4px;
}
.status-hint {
  color: var(--sl-warning);
}
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-2xl);
}
.msg-banner {
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
.msg-banner button {
  font-weight: 600;
  color: inherit;
}
.tab-bar {
  display: flex;
  gap: 2px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-md);
  padding: 3px;
  width: fit-content;
  position: relative;
  overflow: hidden;
}
.tab-indicator {
  position: absolute;
  top: 3px;
  bottom: 3px;
  background: white;
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
}
.tab-btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 8px 18px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
}
.tab-btn.active {
  color: var(--sl-primary);
}
.tab-count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.tab-btn.active .tab-count {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}
.action-bar {
  display: flex;
  gap: var(--sl-space-sm);
}
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  color: var(--sl-text-tertiary);
}
.player-list {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.empty-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--sl-space-2xl);
}
.player-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
}
.player-avatar {
  flex-shrink: 0;
}
.avatar-img {
  width: 36px;
  height: 36px;
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-tertiary);
}
.player-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}
.player-name {
  font-size: 0.9375rem;
  font-weight: 600;
}
.player-uuid {
  font-size: 0.6875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.player-actions {
  flex-shrink: 0;
}
.modal-form {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
</style>
