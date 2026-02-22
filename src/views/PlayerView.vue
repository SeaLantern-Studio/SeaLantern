<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { AlertTriangle } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLBadge from "@components/common/SLBadge.vue";
import SLModal from "@components/common/SLModal.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import { useServerStore } from "@stores/serverStore";
import { useConsoleStore } from "@stores/consoleStore";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry } from "@api/player";
import { TIME, MESSAGES } from "@utils/constants";
import { validatePlayerName, handleError } from "@utils/errorHandler";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { useTabIndicator } from "@composables/useTabIndicator";

const store = useServerStore();
const consoleStore = useConsoleStore();

const activeTab = ref<"online" | "whitelist" | "banned" | "ops">("online");
const { indicatorRef: tabIndicator, updatePosition: updateTabIndicator } =
  useTabIndicator(activeTab);

const whitelist = ref<PlayerEntry[]>([]);
const bannedPlayers = ref<BanEntry[]>([]);
const ops = ref<OpEntry[]>([]);
const onlinePlayers = ref<string[]>([]);

const { loading, withLoading } = useLoading();
const { error, success, showError, showSuccess, clear: clearMessage } = useMessage();

// 监听语言变化，更新 Tab 指示器位置
const localeRef = i18n.getLocaleRef();
watch(localeRef, () => {
  updateTabIndicator();
});

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

onMounted(async () => {
  await store.refreshList();
  // 如果没有当前服务器但有服务器列表，选择第一个
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

  // 找到最后一次服务器启动的位置，从该点开始解析
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
</script>

<template>
  <div class="player-view animate-fade-in-up">
    <div v-if="!selectedServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("players.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="msg-banner error-banner">
        <span>{{ error }}</span>
        <button @click="clearMessage('error')">x</button>
      </div>
      <div v-if="success" class="msg-banner success-banner">
        <span>{{ success }}</span>
      </div>

      <div class="tab-bar">
        <div class="tab-indicator" ref="tabIndicator"></div>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'online' }"
          @click="activeTab = 'online'"
        >
          {{ i18n.t("players.online_players") }}
          <span class="tab-count">{{ onlinePlayers.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'whitelist' }"
          @click="activeTab = 'whitelist'"
        >
          {{ i18n.t("players.whitelist") }} <span class="tab-count">{{ whitelist.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'banned' }"
          @click="activeTab = 'banned'"
        >
          {{ i18n.t("players.banned") }} <span class="tab-count">{{ bannedPlayers.length }}</span>
        </button>
        <button class="tab-btn" :class="{ active: activeTab === 'ops' }" @click="activeTab = 'ops'">
          {{ i18n.t("players.ops") }} <span class="tab-count">{{ ops.length }}</span>
        </button>
      </div>

      <div v-if="activeTab !== 'online'" class="action-bar">
        <SLButton variant="primary" size="sm" :disabled="!isRunning" @click="openAddModal">{{
          getAddLabel()
        }}</SLButton>
        <SLButton variant="ghost" size="sm" @click="loadAll">{{
          i18n.t("common.refresh")
        }}</SLButton>
      </div>

      <div v-if="loading" class="loading-state">
        <SLSpinner />
        <span>{{ i18n.t("common.loading") }}</span>
      </div>

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
              :src="'https://api.rms.net.cn/head/' + name"
              :alt="name"
              class="avatar-img"
              @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
            />
          </div>
          <div class="player-info">
            <span class="player-name">{{ name }}</span>
            <SLBadge :text="i18n.t('players.status_online')" variant="success" />
          </div>
          <div class="player-actions">
            <SLButton variant="ghost" size="sm" @click="handleKick(name)">{{
              i18n.t("players.kick")
            }}</SLButton>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'whitelist'" class="player-list">
        <div v-if="whitelist.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_whitelist") }}</p>
        </div>
        <div v-for="p in whitelist" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img
              :src="'https://api.rms.net.cn/head/' + p.name"
              class="avatar-img"
              @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
            />
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

      <div v-else-if="activeTab === 'banned'" class="player-list">
        <div v-if="bannedPlayers.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_banned") }}</p>
        </div>
        <div v-for="p in bannedPlayers" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img
              :src="'https://api.rms.net.cn/head/' + p.name"
              class="avatar-img"
              @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
            />
          </div>
          <div class="player-info">
            <span class="player-name">{{ p.name }}</span>
            <span class="text-caption"
              >{{ i18n.t("players.reason") }}: {{ p.reason || i18n.t("players.empty") }}</span
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

      <div v-else-if="activeTab === 'ops'" class="player-list">
        <div v-if="ops.length === 0" class="empty-list">
          <p class="text-caption">{{ i18n.t("players.empty_ops") }}</p>
        </div>
        <div v-for="p in ops" :key="p.name" class="player-item glass-card">
          <div class="player-avatar">
            <img
              :src="'https://api.rms.net.cn/head/' + p.name"
              class="avatar-img"
              @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')"
            />
          </div>
          <div class="player-info">
            <span class="player-name">{{ p.name }}</span>
            <span class="text-caption">{{ i18n.t("players.level") }}: {{ p.level }}</span>
          </div>
          <SLBadge text="OP" variant="warning" />
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
          <AlertTriangle
            :size="14"
            style="display: inline; vertical-align: middle; margin-right: 4px"
          />{{ i18n.t("players.server_not_running_hint") }}
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
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
  width: fit-content;
  margin: var(--sl-space-md) 0;
  position: relative;
  overflow: hidden;
}

.tab-indicator {
  position: absolute;
  top: var(--sl-space-xs);
  bottom: var(--sl-space-xs);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  white-space: nowrap;
}

.tab-btn:hover {
  color: var(--sl-text-primary);
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
