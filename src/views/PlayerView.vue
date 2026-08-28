<script setup lang="ts">
// keep-alive 缓存时 onUnmounted 不触发,改用 onActivated/onDeactivated 管理刷新定时器
import { ref, onActivated, onDeactivated, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { playerApi, type PlayerEntry, type BanEntry, type OpEntry } from "@api/player";
import { TIME, MESSAGES, getMessage } from "@utils/constants";
import { validatePlayerName, handleError } from "@utils/errorHandler";
import { i18n } from "@language";
import { useToast } from "cmzya-modern-ui";
import { useLoading } from "@composables/useAsync";
import PlayerTabs from "@components/views/player/PlayerTabs.vue";
import PlayerActionBar from "@components/views/player/PlayerActionBar.vue";
import PlayerList from "@components/views/player/PlayerList.vue";
import PlayerModals from "@components/views/player/PlayerModals.vue";

type PlayerTab = "online" | "whitelist" | "banned" | "ops";

const store = useServerStore();

const activeTab = ref<PlayerTab>("online");

const whitelist = ref<PlayerEntry[]>([]);
const bannedPlayers = ref<BanEntry[]>([]);
const ops = ref<OpEntry[]>([]);
const onlinePlayers = ref<string[]>([]);

const { loading, withLoading } = useLoading();
const toast = useToast();

const showAddModal = ref(false);
const addPlayerName = ref("");
const addBanReason = ref("");
const addLoading = ref(false);

let refreshTimer: ReturnType<typeof setInterval> | null = null;
// 页面隐藏时暂停轮询,避免后台无意义 IPC 开销
let isPageVisible = true;

const selectedServerId = computed(() => store.currentServerId || "");

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

onActivated(async () => {
  isPageVisible = true;
  try {
    await store.refreshList();
  } catch (e) {
    console.warn("Failed to load servers:", e);
  }
  if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  if (store.currentServerId) {
    await store.refreshStatus(store.currentServerId);
    await loadAll();
    await loadOnline();
  }
  startRefresh();
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onDeactivated(() => {
  stopRefresh();
  document.removeEventListener("visibilitychange", handleVisibilityChange);
});

function startRefresh() {
  stopRefresh();
  refreshTimer = setInterval(async () => {
    if (!isPageVisible) return;
    if (selectedServerId.value) {
      await store.refreshStatus(selectedServerId.value);
      await loadAll();
      await loadOnline();
    }
  }, 5000);
}

function stopRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
}

async function refreshNow() {
  if (selectedServerId.value) {
    await store.refreshStatus(selectedServerId.value);
    await loadAll();
    await loadOnline();
  }
}

function handleVisibilityChange() {
  const visible = document.visibilityState === "visible";
  if (visible === isPageVisible) return;
  isPageVisible = visible;
  if (visible) {
    void refreshNow();
    startRefresh();
  } else {
    stopRefresh();
  }
}

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      await store.refreshStatus(store.currentServerId);
      await loadAll();
      await loadOnline();
    }
  },
);

// 加载请求序号:快速切换服务器时丢弃过期响应,避免旧数据覆盖当前服务器
let loadSeq = 0;
// 在线玩家请求单独的序号,因为 loadOnline 可被 handleKick 独立触发
let onlineLoadSeq = 0;

async function loadAll() {
  if (!selectedServerId.value) return;
  const seq = ++loadSeq;
  const sid = selectedServerId.value;
  await withLoading(async () => {
    try {
      // 三个接口互不依赖,并行拉取降低总延迟；只传 server_id,目录由后端
      // 经实例注册表解析（不信任前端 server_path,避免 A/B 服数据错位）
      const [whitelistRes, bannedRes, opsRes] = await Promise.all([
        playerApi.getWhitelist(sid),
        playerApi.getBannedPlayers(sid),
        playerApi.getOps(sid),
      ]);
      // 期间已切换服务器,丢弃这次过期结果
      if (seq !== loadSeq || sid !== selectedServerId.value) return;
      whitelist.value = whitelistRes;
      bannedPlayers.value = bannedRes;
      ops.value = opsRes;
    } catch (e) {
      if (seq !== loadSeq || sid !== selectedServerId.value) return;
      console.error("[players] 加载白名单/封禁/OP 失败:", e);
      toast.error(`加载白名单/封禁/OP 失败: ${handleError(e, "LoadPlayers")}`);
    }
  });
}

async function loadOnline() {
  if (!isRunning.value || !selectedServerId.value) {
    onlinePlayers.value = [];
    return;
  }
  const seq = ++onlineLoadSeq;
  const sid = selectedServerId.value;
  try {
    // 在线玩家来自服务器 list 命令的实时回显,而非解析历史日志
    const names = await playerApi.getOnlinePlayers(sid);
    // 期间已切换服务器或发起新请求,丢弃这次过期结果
    if (seq !== onlineLoadSeq || sid !== selectedServerId.value) return;
    onlinePlayers.value = names;
  } catch (e) {
    if (seq !== onlineLoadSeq || sid !== selectedServerId.value) return;
    console.error("[players] 加载在线玩家失败:", e);
    onlinePlayers.value = [];
    toast.error(`加载在线玩家失败: ${handleError(e, "LoadOnlinePlayers")}`);
  }
}

function openAddModal() {
  addPlayerName.value = "";
  addBanReason.value = "";
  showAddModal.value = true;
}

async function handleAdd() {
  const validation = validatePlayerName(addPlayerName.value);
  if (!validation.valid) {
    toast.error(validation.error || getMessage(MESSAGES.ERROR.INVALID_PLAYER_NAME));
    return;
  }

  if (!isRunning.value) {
    toast.error(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }

  addLoading.value = true;
  try {
    const sid = selectedServerId.value;
    switch (activeTab.value) {
      case "whitelist":
        await playerApi.addToWhitelist(sid, addPlayerName.value);
        toast.success(getMessage(MESSAGES.SUCCESS.WHITELIST_ADDED));
        break;
      case "banned":
        await playerApi.banPlayer(sid, addPlayerName.value, addBanReason.value);
        toast.success(getMessage(MESSAGES.SUCCESS.PLAYER_BANNED));
        break;
      case "ops":
        await playerApi.addOp(sid, addPlayerName.value);
        toast.success(getMessage(MESSAGES.SUCCESS.OP_ADDED));
        break;
    }
    showAddModal.value = false;
    setTimeout(() => {
      loadAll();
    }, TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    toast.error(handleError(e, "AddPlayer"));
  } finally {
    addLoading.value = false;
  }
}

async function handleRemoveWhitelist(name: string) {
  if (!isRunning.value) {
    toast.error(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.removeFromWhitelist(selectedServerId.value, name);
    toast.success(getMessage(MESSAGES.SUCCESS.WHITELIST_REMOVED));
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    toast.error(handleError(e, "RemoveWhitelist"));
  }
}

async function handleUnban(name: string) {
  if (!isRunning.value) {
    toast.error(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.unbanPlayer(selectedServerId.value, name);
    toast.success(getMessage(MESSAGES.SUCCESS.PLAYER_UNBANNED));
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    toast.error(handleError(e, "UnbanPlayer"));
  }
}

async function handleRemoveOp(name: string) {
  if (!isRunning.value) {
    toast.error(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.removeOp(selectedServerId.value, name);
    toast.success(getMessage(MESSAGES.SUCCESS.OP_REMOVED));
    setTimeout(() => loadAll(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    toast.error(handleError(e, "RemoveOp"));
  }
}

async function handleKick(name: string) {
  if (!isRunning.value) {
    toast.error(getMessage(MESSAGES.ERROR.SERVER_NOT_RUNNING));
    return;
  }
  try {
    await playerApi.kickPlayer(selectedServerId.value, name);
    toast.success(`${name} ${getMessage(MESSAGES.SUCCESS.PLAYER_KICKED)}`);
    setTimeout(() => loadOnline(), TIME.SUCCESS_MESSAGE_DURATION);
  } catch (e) {
    toast.error(handleError(e, "KickPlayer"));
  }
}
</script>

<template>
  <div class="player-view animate-stagger-in">
    <div v-if="!selectedServerId" class="player-empty-state">
      <p class="text-body">{{ i18n.t("players.no_server") }}</p>
    </div>

    <template v-else>
      <div class="player-content-layout">
        <PlayerTabs
          v-model="activeTab"
          :onlineCount="onlinePlayers.length"
          :whitelistCount="whitelist.length"
          :bannedCount="bannedPlayers.length"
          :opsCount="ops.length"
        />

        <div class="player-main">
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
            :server-id="selectedServerId"
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
        </div>
      </div>

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

.player-content-layout {
  display: flex;
  align-items: flex-start;
  flex: 1;
  min-height: 0;
}

.player-main {
  flex: 1;
  align-self: stretch;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  min-width: 0;
}
</style>
