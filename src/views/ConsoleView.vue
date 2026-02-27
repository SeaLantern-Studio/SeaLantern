<script setup lang="ts">
import { ref, onMounted, onUnmounted, onActivated, nextTick, computed, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import ConsoleInput from "@components/console/ConsoleInput.vue";
import CommandModal from "@components/console/CommandModal.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { useServerStore } from "@stores/serverStore";
import { serverApi } from "@api/server";
import { settingsApi } from "@api/settings";
import { i18n } from "@language";
import { useLoading } from "@composables/useAsync";
import { SETTINGS_UPDATE_EVENT, type SettingsUpdateEvent } from "@stores/settingsStore";
import type { UnlistenFn } from "@tauri-apps/api/event";

const serverStore = useServerStore();

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: () => void;
  getAllPlainText: () => string;
}

const commandInput = ref("");
const consoleOutputRef = ref<ConsoleOutputExpose | null>(null);
const userScrolledUp = ref(false);
const commandHistory = ref<string[]>([]);
const historyIndex = ref(-1);
const consoleFontSize = ref(13);
const consoleFontFamily = ref("");
const consoleLetterSpacing = ref(0);
const maxLogLines = ref(5000);
const { loading: startLoading, start: startStartLoading, stop: stopStartLoading } = useLoading();
const { loading: stopLoading, start: startStopLoading, stop: stopStopLoading } = useLoading();
let unlistenLogLine: UnlistenFn | null = null;

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

const serverStatus = computed(() => serverStore.statuses[serverId.value]?.status || "Stopped");

const isRunning = computed(() => serverStatus.value === "Running");
const isStopped = computed(
  () => serverStatus.value === "Stopped" || serverStatus.value === "Error" || !serverStatus.value,
);
const isStopping = computed(() => serverStatus.value === "Stopping");
const isStarting = computed(() => serverStatus.value === "Starting");

onMounted(async () => {
  await loadConsoleSettings();
  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);

  await serverStore.refreshList();
  // 如果没有当前服务器但有服务器列表，选择第一个
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (serverId.value) {
    await serverStore.refreshStatus(serverId.value);
    await syncLogsOnce(serverId.value);
  }
  unlistenLogLine = await serverApi.onLogLine(({ server_id, line }) => {
    const sid = serverId.value;
    if (!sid || server_id !== sid) return;
    consoleOutputRef.value?.appendLines([line]);
  });
  nextTick(() => doScroll());
});

onUnmounted(() => {
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);
  if (unlistenLogLine) {
    unlistenLogLine();
    unlistenLogLine = null;
  }
});

onActivated(async () => {
  await loadConsoleSettings();
});

watch(
  () => serverId.value,
  async (sid) => {
    if (!sid) return;
    await serverStore.refreshStatus(sid);
    await syncLogsOnce(sid);
    userScrolledUp.value = false;
    nextTick(() => doScroll());
  },
);

async function syncLogsOnce(sid: string) {
  consoleOutputRef.value?.clear();
  try {
    const lines = await serverApi.getLogs(sid, 0, Math.max(1, maxLogLines.value));
    consoleOutputRef.value?.appendLines(lines);
  } catch (_e) {}
}

async function loadConsoleSettings() {
  try {
    const settings = await settingsApi.get();
    applyConsoleSettings(settings);
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

function applyConsoleSettings(settings: {
  console_font_size: number;
  console_font_family: string;
  console_letter_spacing: number;
  max_log_lines: number;
}) {
  consoleFontSize.value = settings.console_font_size;
  consoleFontFamily.value = settings.console_font_family || "";
  consoleLetterSpacing.value = settings.console_letter_spacing || 0;
  maxLogLines.value = Math.max(100, settings.max_log_lines || 5000);
}

function handleSettingsUpdate(event: CustomEvent<SettingsUpdateEvent>) {
  applyConsoleSettings(event.detail.settings);
}

async function sendCommand(cmd?: string) {
  const command = (cmd || commandInput.value).trim();
  const sid = serverId.value;
  if (!command || !sid) return;
  consoleOutputRef.value?.appendLines(["> " + command]);
  commandHistory.value.push(command);
  if (commandHistory.value.length > 500) {
    commandHistory.value.splice(0, commandHistory.value.length - 500);
  }
  historyIndex.value = -1;
  try {
    await serverApi.sendCommand(sid, command);
  } catch (e) {
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
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
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
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
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
  } finally {
    stopStopLoading();
  }
}

async function exportLogs() {
  const text = consoleOutputRef.value?.getAllPlainText() || "";
  if (!text.trim()) return;
  const lineCount = text.split("\n").length;
  try {
    await navigator.clipboard.writeText(text);
    consoleOutputRef.value?.appendLines([
      "[Sea Lantern] 日志已复制到剪贴板（" + lineCount + " 行）",
    ]);
  } catch (_e) {
    consoleOutputRef.value?.appendLines(["[Sea Lantern] 复制日志到剪贴板失败"]);
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
  consoleOutputRef.value?.clear();
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
        <div class="action-group primary-actions">
          <SLButton
            v-if="isRunning || isStarting"
            variant="danger"
            size="sm"
            :loading="stopLoading"
            :disabled="isStopping || stopLoading"
            @click="handleStop"
          >
            {{ isStarting ? i18n.t("home.stop") : i18n.t("home.stop") }}
          </SLButton>
          <SLButton
            v-else
            variant="primary"
            size="sm"
            :loading="startLoading"
            :disabled="isStopping || startLoading"
            @click="handleStart"
          >
            {{ i18n.t("home.start") }}
          </SLButton>
        </div>
        <div class="action-group secondary-actions">
          <SLButton variant="secondary" size="sm" @click="exportLogs">{{
            i18n.t("console.copy_log")
          }}</SLButton>
          <SLButton variant="ghost" size="sm" @click="handleClearLogs">{{
            i18n.t("console.clear_log")
          }}</SLButton>
        </div>
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
        :consoleFontSize="consoleFontSize"
        :consoleFontFamily="consoleFontFamily"
        :consoleLetterSpacing="consoleLetterSpacing"
        :maxLogLines="maxLogLines"
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
