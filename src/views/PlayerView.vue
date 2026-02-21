<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { AlertTriangle } from "lucide-vue-next";
import { useRoute } from "vue-router";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLInput from "../components/common/SLInput.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLBadge from "../components/common/SLBadge.vue";
import SLModal from "../components/common/SLModal.vue";
import SLSpinner from "../components/common/SLSpinner.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry } from "../api/player";
import { serverApi } from "../api/server";
import { TIME, MESSAGES } from "../utils/constants";
import { validatePlayerName, handleError } from "../utils/errorHandler";
import { i18n } from "../language";
import { useMessage } from "../composables/useMessage";
import { useLoading } from "../composables/useAsync";
import { useTabIndicator } from "../composables/useTabIndicator";

const route = useRoute();
const store = useServerStore();
const consoleStore = useConsoleStore();

const selectedServerId = ref("");
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

const serverOptions = computed(() => store.servers.map((s) => ({ label: s.name, value: s.id })));

const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === selectedServerId.value);
  return server?.path || "";
});

const isRunning = computed(() => {
  return store.statuses[selectedServerId.value]?.status === "Running";
});

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) selectedServerId.value = routeId;
  else if (store.currentServerId) selectedServerId.value = store.currentServerId;
  else if (store.servers.length > 0) selectedServerId.value = store.servers[0].id;

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

watch(selectedServerId, async () => {
  if (selectedServerId.value) {
    await store.refreshStatus(selectedServerId.value);
    await loadAll();
    parseOnlinePlayers();
  }
});

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
      return "添加白名单";
    case "banned":
      return "封禁玩家";
    case "ops":
      return "添加管理员";
    default:
      return "添加";
  }
}

function selectTab(tab: "online" | "whitelist" | "banned" | "ops") {
  activeTab.value = tab;
  updateTabIndicator();
}
</script>

<template>
  <div class="player-view animate-fade-in-up">
    <div class="player-header">
      <div class="server-picker">
        <SLSelect
          label="选择服务器"
          :options="serverOptions"
          v-model="selectedServerId"
          placeholder="选择服务器"
        />
      </div>
      <div v-if="selectedServerId" class="server-status">
        <SLBadge
          :text="isRunning ? '运行中' : '已停止'"
          :variant="isRunning ? 'success' : 'neutral'"
        />
        <span v-if="!isRunning" class="status-hint text-caption"
          >玩家管理需要服务器运行中才能操作</span
        >
      </div>
    </div>

    <div v-if="!selectedServerId" class="empty-state">
      <p class="text-body">请选择一个服务器</p>
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
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'online' }"
          @click="activeTab = 'online'"
        >
          在线玩家 <span class="tab-count">{{ onlinePlayers.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'whitelist' }"
          @click="activeTab = 'whitelist'"
        >
          白名单 <span class="tab-count">{{ whitelist.length }}</span>
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'banned' }"
          @click="activeTab = 'banned'"
        >
          封禁列表 <span class="tab-count">{{ bannedPlayers.length }}</span>
        </button>
        <button class="tab-btn" :class="{ active: activeTab === 'ops' }" @click="activeTab = 'ops'">
          管理员 <span class="tab-count">{{ ops.length }}</span>
        </button>
      </div>

      <div v-if="activeTab !== 'online'" class="action-bar">
        <SLButton variant="primary" size="sm" :disabled="!isRunning" @click="openAddModal">{{
          getAddLabel()
        }}</SLButton>
        <SLButton variant="ghost" size="sm" @click="loadAll">刷新</SLButton>
      </div>

      <div v-if="loading" class="loading-state">
        <SLSpinner />
        <span>加载中...</span>
      </div>

      <div v-else-if="activeTab === 'online'" class="player-list">
        <div v-if="!isRunning" class="empty-list"><p class="text-caption">服务器未运行</p></div>
        <div v-else-if="onlinePlayers.length === 0" class="empty-list">
          <p class="text-caption">当前没有玩家在线</p>
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
            <SLBadge text="在线" variant="success" />
          </div>
          <div class="player-actions">
            <SLButton variant="ghost" size="sm" @click="handleKick(name)">踢出</SLButton>
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'whitelist'" class="player-list">
        <div v-if="whitelist.length === 0" class="empty-list">
          <p class="text-caption">白名单为空</p>
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
              >移除</SLButton
            >
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'banned'" class="player-list">
        <div v-if="bannedPlayers.length === 0" class="empty-list">
          <p class="text-caption">封禁列表为空</p>
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
            <span class="text-caption">原因: {{ p.reason || "无" }}</span>
          </div>
          <SLBadge text="封禁" variant="error" />
          <div class="player-actions">
            <SLButton variant="ghost" size="sm" :disabled="!isRunning" @click="handleUnban(p.name)"
              >解封</SLButton
            >
          </div>
        </div>
      </div>

      <div v-else-if="activeTab === 'ops'" class="player-list">
        <div v-if="ops.length === 0" class="empty-list">
          <p class="text-caption">管理员列表为空</p>
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
            <span class="text-caption">等级: {{ p.level }}</span>
          </div>
          <SLBadge text="OP" variant="warning" />
          <div class="player-actions">
            <SLButton
              variant="ghost"
              size="sm"
              :disabled="!isRunning"
              @click="handleRemoveOp(p.name)"
              >取消OP</SLButton
            >
          </div>
        </div>
      </div>
    </template>

    <SLModal :visible="showAddModal" :title="getAddLabel()" @close="showAddModal = false">
      <div class="modal-form">
        <SLInput label="玩家名称" placeholder="输入玩家游戏ID" v-model="addPlayerName" />
        <SLInput
          v-if="activeTab === 'banned'"
          label="封禁原因（可选）"
          placeholder="输入原因"
          v-model="addBanReason"
        />
        <p v-if="!isRunning" class="text-error" style="font-size: 0.8125rem">
          <AlertTriangle
            :size="14"
            style="display: inline; vertical-align: middle; margin-right: 4px"
          />服务器未运行，无法发送命令
        </p>
      </div>
      <template #footer>
        <SLButton variant="secondary" @click="showAddModal = false">取消</SLButton>
        <SLButton variant="primary" :loading="addLoading" :disabled="!isRunning" @click="handleAdd"
          >确认</SLButton
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
}
.tab-btn.active {
  background: var(--sl-surface);
  color: var(--sl-primary);
  box-shadow: var(--sl-shadow-sm);
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
