<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from "vue";
import { useRoute } from "vue-router";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { serverApi } from "../api/server";
import { settingsApi } from "../api/settings";
import { i18n } from "../locales";
import type { ServerCommand } from "../types/server";

// 导入拆分后的组件
import ConsoleToolbar from "../components/console/ConsoleToolbar.vue";
import ConsoleCommands from "../components/console/ConsoleCommands.vue";
import ConsoleOutput from "../components/console/ConsoleOutput.vue";
import ConsoleInput from "../components/console/ConsoleInput.vue";
import CommandModal from "../components/console/CommandModal.vue";

const route = useRoute();
const serverStore = useServerStore();
const consoleStore = useConsoleStore();

const userScrolledUp = ref(false);
const consoleFontSize = ref(13);
const startLoading = ref(false);
const stopLoading = ref(false);
const isPolling = ref(false); // 添加轮询锁
let pollTimer: ReturnType<typeof setInterval> | null = null;

// 自定义指令相关
const showCommandModal = ref(false);
const editingCommand = ref<ServerCommand | null>(null);
const commandName = ref("");
const commandText = ref("");
const commandModalTitle = ref("");
const commandLoading = ref(false);

// 优先使用serverStore.currentServerId，确保与侧栏同步
const serverId = computed(() => {
  return (
    serverStore.currentServerId || consoleStore.activeServerId || (route.params.id as string) || ""
  );
});

const currentLogs = computed(() => consoleStore.logs[serverId.value] || []);

const serverStatus = computed(() => serverStore.statuses[serverId.value]?.status || "Stopped");

const isRunning = computed(() => serverStatus.value === "Running");
const isStopped = computed(() => serverStatus.value === "Stopped");
const isStopping = computed(() => serverStatus.value === "Stopping");

const currentServerCommands = computed(() => {
  const server = serverStore.servers.find((s) => s.id === serverId.value);
  return server?.commands || [];
});

const serverName = computed(() => {
  return serverStore.servers.find((s) => s.id === serverId.value)?.name || "";
});

watch(
  () => serverId.value,
  async (newServerId, oldServerId) => {
    if (newServerId && newServerId !== oldServerId) {
      // 确保consoleStore与serverStore保持同步
      consoleStore.setActiveServer(newServerId);
      // 同时更新serverStore的当前服务器，确保双向同步
      if (newServerId !== serverStore.currentServerId) {
        serverStore.setCurrentServer(newServerId);
      }
      await serverStore.refreshStatus(newServerId);
      userScrolledUp.value = false;
      nextTick(() => doScroll());
    }
  },
);

// 直接监听serverStore.currentServerId的变化，确保侧栏选择能立即同步到控制台
watch(
  () => serverStore.currentServerId,
  async (newServerId) => {
    if (newServerId && newServerId !== consoleStore.activeServerId) {
      consoleStore.setActiveServer(newServerId);
      await serverStore.refreshStatus(newServerId);
      userScrolledUp.value = false;
      nextTick(() => doScroll());
    }
  },
);

onMounted(async () => {
  // Load console font size from settings
  try {
    const settings = await settingsApi.get();
    consoleFontSize.value = settings.console_font_size;
  } catch (e) {
    console.error("Failed to load settings:", e);
  }

  await serverStore.refreshList();
  if (serverId.value) {
    consoleStore.setActiveServer(serverId.value);
    serverStore.setCurrentServer(serverId.value);
    await serverStore.refreshStatus(serverId.value);
  }
  startPolling();
  nextTick(() => doScroll());
});

onUnmounted(() => {
  stopPolling();
});

function startPolling() {
  stopPolling();
  pollTimer = setInterval(async () => {
    // 防止并发执行
    if (isPolling.value) return;
    isPolling.value = true;

    try {
      const sid = serverId.value;
      if (!sid) return;
      const cursor = consoleStore.getLogCursor(sid);
      try {
        const newLines = await serverApi.getLogs(sid, cursor);
        if (newLines.length > 0) {
          consoleStore.appendLogs(sid, newLines);
          consoleStore.setLogCursor(sid, cursor + newLines.length);
        }
      } catch (_e) {}
      await serverStore.refreshStatus(sid);
    } finally {
      isPolling.value = false;
    }
  }, 800);
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

async function sendCommand(cmd: string) {
  const command = cmd.trim();
  const sid = serverId.value;
  if (!command || !sid) return;
  consoleStore.appendLocal(sid, "> " + command);
  try {
    await serverApi.sendCommand(sid, command);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  }
  userScrolledUp.value = false;
  doScroll();
}

function doScroll() {
  nextTick(() => {
    // 滚动逻辑已移至ConsoleOutput组件
  });
}

async function handleStart() {
  const sid = serverId.value;
  if (!sid) return;
  startLoading.value = true;
  try {
    await serverApi.start(sid);
    await serverStore.refreshStatus(sid);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  } finally {
    startLoading.value = false;
  }
}

async function handleStop() {
  const sid = serverId.value;
  if (!sid) return;
  stopLoading.value = true;
  try {
    await serverApi.stop(sid);
    await serverStore.refreshStatus(sid);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  } finally {
    stopLoading.value = false;
  }
}

async function exportLogs() {
  const logs = currentLogs.value;
  if (logs.length === 0) return;
  // Copy to clipboard as fallback
  const text = logs.join("\n");
  try {
    await navigator.clipboard.writeText(text);
    consoleStore.appendLocal(
      serverId.value,
      "[Sea Lantern] Logs copied to clipboard (" + logs.length + " lines)",
    );
  } catch (_e) {
    consoleStore.appendLocal(serverId.value, "[Sea Lantern] Failed to copy logs");
  }
}

function getStatusClass(): string {
  const s = serverStore.statuses[serverId.value]?.status;
  return s === "Running"
    ? "running"
    : s === "Starting"
      ? "starting"
      : s === "Stopping"
        ? "stopping"
        : "stopped";
}

function getStatusText(): string {
  const s = serverStore.statuses[serverId.value]?.status;
  return s === "Running"
    ? i18n.t("console.running")
    : s === "Starting"
      ? i18n.t("console.starting")
      : s === "Stopping"
        ? i18n.t("console.stopping")
        : i18n.t("console.stopped");
}

function handleClearLogs() {
  const sid = serverId.value;
  console.log("[清屏] serverId:", sid);
  console.log("[清屏] 当前日志数量:", currentLogs.value.length);
  if (!sid) {
    console.log("[清屏] serverId 为空，取消操作");
    return;
  }
  consoleStore.clearLogs(sid);
  console.log("[清屏] 清空后日志数量:", currentLogs.value.length);
  userScrolledUp.value = false;
}

// 自定义指令相关方法
function openAddCommandModal() {
  editingCommand.value = null;
  commandName.value = "";
  commandText.value = "";
  commandModalTitle.value = i18n.t("console.add_custom_command");
  showCommandModal.value = true;
}

function openEditCommandModal(cmd: ServerCommand) {
  editingCommand.value = cmd;
  commandName.value = cmd.name;
  commandText.value = cmd.command;
  commandModalTitle.value = i18n.t("console.edit_custom_command");
  showCommandModal.value = true;
}

async function saveCommand() {
  const sid = serverId.value;
  if (!sid || !commandName.value.trim() || !commandText.value.trim()) return;

  commandLoading.value = true;
  try {
    if (editingCommand.value) {
      // 更新现有指令
      await serverApi.updateServerCommand(
        sid,
        editingCommand.value.id,
        commandName.value.trim(),
        commandText.value.trim(),
      );
    } else {
      // 添加新指令
      await serverApi.addServerCommand(sid, commandName.value.trim(), commandText.value.trim());
    }
    // 刷新服务器列表以获取更新的指令
    await serverStore.refreshList();
    showCommandModal.value = false;
  } catch (e) {
    console.error("保存指令失败:", e);
    consoleStore.appendLocal(sid, "[ERROR] 保存自定义指令失败: " + String(e));
  } finally {
    commandLoading.value = false;
  }
}

async function deleteCommand(cmd: ServerCommand) {
  const sid = serverId.value;
  if (!sid) return;

  try {
    await serverApi.deleteServerCommand(sid, cmd.id);
    // 刷新服务器列表以获取更新的指令
    await serverStore.refreshList();
    // 关闭模态框
    showCommandModal.value = false;
  } catch (e) {
    console.error("删除指令失败:", e);
    consoleStore.appendLocal(sid, "[ERROR] 删除自定义指令失败: " + String(e));
  }
}
</script>

<template>
  <div class="console-view animate-fade-in-up">
    <!-- 工具栏 -->
    <ConsoleToolbar
      :serverId="serverId"
      :serverName="serverName"
      :statusClass="getStatusClass()"
      :statusText="getStatusText()"
      :isRunning="isRunning"
      :isStopped="isStopped"
      :isStopping="isStopping"
      :startLoading="startLoading"
      :stopLoading="stopLoading"
      @start="handleStart"
      @stop="handleStop"
      @export="exportLogs"
      @clear="handleClearLogs"
    />

    <div v-if="!serverId" class="no-server">
      <p class="text-body">{{ i18n.t("home.no_servers") }}</p>
    </div>

    <template v-else>
      <!-- 快捷指令和自定义指令部分 -->
      <ConsoleCommands
        :serverId="serverId"
        :currentServerCommands="currentServerCommands"
        @sendCommand="sendCommand"
        @openAddCommandModal="openAddCommandModal"
        @openEditCommandModal="openEditCommandModal"
      />

      <!-- 控制台输出部分 -->
      <ConsoleOutput
        :logs="currentLogs"
        :consoleFontSize="consoleFontSize"
        :userScrolledUp="userScrolledUp"
        @scroll="(value) => (userScrolledUp = value)"
        @scrollToBottom="
          userScrolledUp = false;
          doScroll();
        "
      />

      <!-- 控制台输入部分 -->
      <ConsoleInput :consoleFontSize="consoleFontSize" @sendCommand="sendCommand" />

      <!-- 自定义指令模态框 -->
      <CommandModal
        :visible="showCommandModal"
        :title="commandModalTitle"
        :editingCommand="editingCommand"
        :commandName="commandName"
        :commandText="commandText"
        :loading="commandLoading"
        @close="showCommandModal = false"
        @save="saveCommand"
        @delete="deleteCommand"
        @updateName="(value) => (commandName = value)"
        @updateText="(value) => (commandText = value)"
      />
    </template>
  </div>
</template>

<style scoped>
.console-view {
  display: flex;
  flex-direction: column;
  height: calc(100vh - var(--sl-header-height) - var(--sl-space-lg) * 2);
  gap: var(--sl-space-sm);
  position: relative;
}

.no-server {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
