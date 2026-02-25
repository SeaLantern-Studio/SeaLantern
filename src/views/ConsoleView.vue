<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import ConsoleInput from "@components/console/ConsoleInput.vue";
import CommandModal from "@components/console/CommandModal.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { useServerStore } from "@stores/serverStore";
import { useConsoleStore } from "@stores/consoleStore";
import { serverApi } from "@api/server";
import { settingsApi } from "@api/settings";
import { i18n } from "@language";
import { useLoading } from "@composables/useAsync";

const serverStore = useServerStore();
const consoleStore = useConsoleStore();

const commandInput = ref("");
const consoleOutputRef = ref<InstanceType<typeof ConsoleOutput> | null>(null);
const userScrolledUp = ref(false);
const commandHistory = ref<string[]>([]);
const historyIndex = ref(-1);
const consoleFontSize = ref(13);
const { loading: startLoading, start: startStartLoading, stop: stopStartLoading } = useLoading();
const { loading: stopLoading, start: startStopLoading, stop: stopStopLoading } = useLoading();
const isPolling = ref(false);
let pollTimer: ReturnType<typeof setInterval> | null = null;

const showCommandModal = ref(false);
const commandModalTitle = ref("");
const editingCommand = ref<import("@type/server").ServerCommand | null>(null);
const commandName = ref("");
const commandText = ref("");
const commandLoading = ref(false);

const quickCommands = computed(() => [
  { label: i18n.t("common.command_day"), cmd: "time set day" },
  { label: i18n.t("common.command_night"), cmd: "time set night" },
  { label: i18n.t("common.command_clear"), cmd: "weather clear" },
  { label: i18n.t("common.command_rain"), cmd: "weather rain" },
  { label: i18n.t("common.command_save"), cmd: "save-all" },
  { label: i18n.t("common.command_list"), cmd: "list" },
  { label: "TPS", cmd: "tps" },
  { label: i18n.t("common.command_keep_inventory_on"), cmd: "gamerule keepInventory true" },
  { label: i18n.t("common.command_keep_inventory_off"), cmd: "gamerule keepInventory false" },
  { label: i18n.t("common.command_mob_griefing_off"), cmd: "gamerule mobGriefing false" },
]);

const serverId = computed(() => serverStore.currentServerId || "");

const currentLogs = computed(() => consoleStore.logs[serverId.value] || []);

const serverStatus = computed(() => serverStore.statuses[serverId.value]?.status || "Stopped");

const isRunning = computed(() => serverStatus.value === "Running");
const isStopped = computed(() => serverStatus.value === "Stopped");
const isStopping = computed(() => serverStatus.value === "Stopping");

watch(
  () => currentLogs.value.length,
  () => {
    if (!userScrolledUp.value) doScroll();
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
  // 如果没有当前服务器但有服务器列表，选择第一个
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (serverId.value) {
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

async function sendCommand(cmd?: string) {
  const command = (cmd || commandInput.value).trim();
  const sid = serverId.value;
  if (!command || !sid) return;
  consoleStore.appendLocal(sid, "> " + command);
  commandHistory.value.push(command);
  if (commandHistory.value.length > 500) {
    commandHistory.value.splice(0, commandHistory.value.length - 500);
  }
  historyIndex.value = -1;
  try {
    await serverApi.sendCommand(sid, command);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  }
  commandInput.value = "";
  userScrolledUp.value = false;
  doScroll();
}

function doScroll() {
  consoleOutputRef.value?.doScroll();
}

async function handleStart() {
  const sid = serverId.value;
  if (!sid) return;
  startStartLoading();
  try {
    await serverApi.start(sid);
    await serverStore.refreshStatus(sid);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  } finally {
    stopStartLoading();
  }
}

async function handleStop() {
  const sid = serverId.value;
  if (!sid) return;
  startStopLoading();
  try {
    await serverApi.stop(sid);
    await serverStore.refreshStatus(sid);
  } catch (e) {
    consoleStore.appendLocal(sid, "[ERROR] " + String(e));
  } finally {
    stopStopLoading();
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
  switch (s) {
    case "Running":
      return i18n.t("common.server_status_running");
    case "Starting":
      return i18n.t("common.server_status_starting");
    case "Stopping":
      return i18n.t("common.server_status_stopping");
    case "Error":
      return i18n.t("common.server_status_error");
    default:
      return i18n.t("common.server_status_stopped");
  }
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

function saveCommand() {
  console.warn("saveCommand not implemented");
  showCommandModal.value = false;
}

function deleteCommand(_cmd: import("@type/server").ServerCommand) {
  console.warn("deleteCommand not implemented");
  showCommandModal.value = false;
}
</script>

<template>
  <div class="console-view animate-fade-in-up">
    <div class="console-toolbar">
      <div class="toolbar-left">
        <div v-if="serverId" class="server-name-display">
          {{
            serverStore.servers.find((s) => s.id === serverId)?.name || i18n.t("console.no_server")
          }}
        </div>
        <div v-else class="server-name-display">{{ i18n.t("console.no_server") }}</div>
        <div v-if="serverId" class="status-indicator" :class="getStatusClass()">
          <span class="status-dot"></span>
          <span class="status-label">{{ getStatusText() }}</span>
        </div>
      </div>
      <div class="toolbar-right">
        <SLButton
          variant="primary"
          size="sm"
          :loading="startLoading"
          :disabled="isRunning || isStopping || startLoading"
          @click="handleStart"
          >{{ i18n.t("home.start") }}</SLButton
        >
        <SLButton
          variant="danger"
          size="sm"
          :loading="stopLoading"
          :disabled="isStopped || isStopping || stopLoading"
          @click="handleStop"
          >{{ i18n.t("home.stop") }}</SLButton
        >
        <SLButton variant="secondary" size="sm" @click="exportLogs">{{
          i18n.t("console.copy_log")
        }}</SLButton>
        <SLButton variant="ghost" size="sm" @click="handleClearLogs">{{
          i18n.t("console.clear_log")
        }}</SLButton>
      </div>
    </div>

    <div v-if="!serverId" class="no-server">
      <p class="text-body">{{ i18n.t("console.create_server_first") }}</p>
    </div>

    <template v-else>
      <div class="quick-commands">
        <span class="quick-label">{{ i18n.t("console.quick") }}</span>
        <div class="quick-groups">
          <div
            v-for="cmd in quickCommands"
            :key="cmd.cmd"
            class="quick-btn"
            @click="sendCommand(cmd.cmd)"
            :title="cmd.cmd"
          >
            {{ cmd.label }}
          </div>
        </div>
      </div>

      <!-- 控制台输出部分 -->
      <ConsoleOutput
        ref="consoleOutputRef"
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

<style src="@styles/views/ConsoleView.css" scoped></style>
