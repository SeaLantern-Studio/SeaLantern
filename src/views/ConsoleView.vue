<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from "vue";
import { useRoute } from "vue-router";
import SLButton from "../components/common/SLButton.vue";
import SLSelect from "../components/common/SLSelect.vue";
import ConsoleInput from "../components/console/ConsoleInput.vue";
import CommandModal from "../components/console/CommandModal.vue";
import ConsoleOutput from "../components/console/ConsoleOutput.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { serverApi } from "../api/server";
import { playerApi } from "../api/player";
import { settingsApi } from "../api/settings";
import { i18n } from "../language";
import { useLoading } from "../composables/useAsync";
import { getStatusClass, getStatusText } from "../utils/serverStatus";

const route = useRoute();
const serverStore = useServerStore();
const consoleStore = useConsoleStore();

const commandInput = ref("");
const logContainer = ref<HTMLElement | null>(null);
const userScrolledUp = ref(false);
const showSuggestions = ref(false);
const suggestionIndex = ref(0);
const commandHistory = ref<string[]>([]);
const historyIndex = ref(-1);
const consoleFontSize = ref(13);
const { loading: startLoading, start: startStartLoading, stop: stopStartLoading } = useLoading();
const { loading: stopLoading, start: startStopLoading, stop: stopStopLoading } = useLoading();
const isPolling = ref(false);
let pollTimer: ReturnType<typeof setInterval> | null = null;

const showCommandModal = ref(false);
const commandModalTitle = ref("");
const editingCommand = ref<import("../types/server").ServerCommand | null>(null);
const commandName = ref("");
const commandText = ref("");
const commandLoading = ref(false);

const allCommands = [
  "help",
  "list",
  "stop",
  "say",
  "time set day",
  "time set night",
  "time set noon",
  "weather clear",
  "weather rain",
  "weather thunder",
  "gamemode survival",
  "gamemode creative",
  "gamemode adventure",
  "gamemode spectator",
  "difficulty peaceful",
  "difficulty easy",
  "difficulty normal",
  "difficulty hard",
  "give",
  "tp",
  "teleport",
  "kill",
  "kick",
  "ban",
  "pardon",
  "op",
  "deop",
  "whitelist add",
  "whitelist remove",
  "whitelist list",
  "gamerule keepInventory true",
  "gamerule keepInventory false",
  "gamerule doDaylightCycle true",
  "gamerule doDaylightCycle false",
  "gamerule mobGriefing true",
  "gamerule mobGriefing false",
  "save-all",
  "tps",
  "plugins",
  "version",
];

const quickCommands = [
  { label: "白天", cmd: "time set day" },
  { label: "夜晚", cmd: "time set night" },
  { label: "晴天", cmd: "weather clear" },
  { label: "下雨", cmd: "weather rain" },
  { label: "保存", cmd: "save-all" },
  { label: "玩家列表", cmd: "list" },
  { label: "TPS", cmd: "tps" },
  { label: "保留物品 开", cmd: "gamerule keepInventory true" },
  { label: "保留物品 关", cmd: "gamerule keepInventory false" },
  { label: "怪物破坏 关", cmd: "gamerule mobGriefing false" },
];

const filteredSuggestions = computed(() => {
  const input = commandInput.value.trim().toLowerCase();
  if (!input) return [];
  return allCommands
    .filter((c) => c.toLowerCase().startsWith(input) && c.toLowerCase() !== input)
    .slice(0, 8);
});

const serverId = computed(
  () =>
    consoleStore.activeServerId || serverStore.currentServerId || (route.params.id as string) || "",
);

const currentLogs = computed(() => consoleStore.logs[serverId.value] || []);

const serverOptions = computed(() =>
  serverStore.servers.map((s) => ({
    label: s.name + " (" + s.id.substring(0, 8) + ")",
    value: s.id,
  })),
);

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

function switchServer(id: string | number) {
  consoleStore.setActiveServer(String(id));
  serverStore.setCurrentServer(String(id));
  userScrolledUp.value = false;
  nextTick(() => doScroll());
}

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
  showSuggestions.value = false;
  userScrolledUp.value = false;
  doScroll();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    if (showSuggestions.value && filteredSuggestions.value.length > 0) {
      commandInput.value = filteredSuggestions.value[suggestionIndex.value];
      showSuggestions.value = false;
    } else {
      sendCommand();
    }
    return;
  }
  if (e.key === "Tab") {
    e.preventDefault();
    if (filteredSuggestions.value.length > 0) {
      commandInput.value = filteredSuggestions.value[suggestionIndex.value];
      showSuggestions.value = false;
    }
    return;
  }
  if (e.key === "ArrowUp") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value > 0) suggestionIndex.value--;
    else if (
      commandHistory.value.length > 0 &&
      historyIndex.value < commandHistory.value.length - 1
    ) {
      historyIndex.value++;
      commandInput.value =
        commandHistory.value[commandHistory.value.length - 1 - historyIndex.value];
    }
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (showSuggestions.value && suggestionIndex.value < filteredSuggestions.value.length - 1)
      suggestionIndex.value++;
    else if (historyIndex.value > 0) {
      historyIndex.value--;
      commandInput.value =
        commandHistory.value[commandHistory.value.length - 1 - historyIndex.value];
    } else {
      historyIndex.value = -1;
      commandInput.value = "";
    }
    return;
  }
  if (e.key === "Escape") {
    showSuggestions.value = false;
    return;
  }
  nextTick(() => {
    showSuggestions.value =
      commandInput.value.trim().length > 0 && filteredSuggestions.value.length > 0;
    suggestionIndex.value = 0;
  });
}

function doScroll() {
  nextTick(() => {
    if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight;
  });
}

function handleScroll() {
  if (!logContainer.value) return;
  const el = logContainer.value;
  userScrolledUp.value = el.scrollHeight - el.scrollTop - el.clientHeight > 40;
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
  return s === "Running"
    ? "Running"
    : s === "Starting"
      ? "Starting"
      : s === "Stopping"
        ? "Stopping"
        : "Stopped";
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

function deleteCommand(_cmd: import("../types/server").ServerCommand) {
  console.warn("deleteCommand not implemented");
  showCommandModal.value = false;
}
</script>

<template>
  <div class="console-view animate-fade-in-up">
    <div class="console-toolbar">
      <div class="toolbar-left">
        <div v-if="serverOptions.length > 0" class="server-selector">
          <SLSelect
            :options="serverOptions"
            :modelValue="serverId"
            placeholder="选择服务器"
            @update:modelValue="switchServer"
          />
        </div>
        <div v-else class="server-name-display">暂无服务器</div>
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
          >启动</SLButton
        >
        <SLButton
          variant="danger"
          size="sm"
          :loading="stopLoading"
          :disabled="isStopped || isStopping || stopLoading"
          @click="handleStop"
          >停止</SLButton
        >
        <SLButton variant="secondary" size="sm" @click="exportLogs">复制日志</SLButton>
        <SLButton variant="ghost" size="sm" @click="handleClearLogs">清屏</SLButton>
      </div>
    </div>

    <div v-if="!serverId" class="no-server">
      <p class="text-body">请先创建并选择一个服务器</p>
    </div>

    <template v-else>
      <div class="quick-commands">
        <span class="quick-label">快捷:</span>
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
.console-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
}
.toolbar-right {
  display: flex;
  gap: var(--sl-space-xs);
}
.server-selector {
  min-width: 240px;
}
.server-name-display {
  font-weight: 600;
}
.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 2px 10px;
  border-radius: var(--sl-radius-full);
  font-size: 0.8125rem;
  font-weight: 500;
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.status-indicator.running {
  background: rgba(34, 197, 94, 0.1);
  color: var(--sl-success);
}
.status-indicator.running .status-dot {
  background: var(--sl-success);
}
.status-indicator.stopped {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
}
.status-indicator.stopped .status-dot {
  background: var(--sl-text-tertiary);
}
.status-indicator.starting,
.status-indicator.stopping {
  background: rgba(245, 158, 11, 0.1);
  color: var(--sl-warning);
}
.status-indicator.starting .status-dot,
.status-indicator.stopping .status-dot {
  background: var(--sl-warning);
}
.no-server {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.quick-commands {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-xs) var(--sl-space-sm);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
  overflow-x: auto;
}
.quick-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  white-space: nowrap;
}
.quick-groups {
  display: flex;
  gap: 4px;
}
.quick-btn {
  padding: 3px 10px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  background: var(--sl-bg-secondary);
  white-space: nowrap;
  transition: all var(--sl-transition-fast);
}
.quick-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}
.console-output {
  flex: 1;
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-md);
  overflow-y: auto;
  font-family: var(--sl-font-mono);
  line-height: 1.7;
  color: var(--sl-text-primary);
  min-height: 0;
  user-select: text;
  -webkit-user-select: text;
  cursor: text;
}
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}
.log-error {
  color: var(--sl-error);
  font-weight: 500;
}
.log-warn {
  color: var(--sl-warning);
  font-weight: 500;
}
.log-command {
  color: var(--sl-info);
  font-weight: 600;
}
.log-system {
  color: var(--sl-success);
  font-style: italic;
}
.log-empty {
  color: var(--sl-text-tertiary);
  font-style: italic;
}
.scroll-btn {
  position: absolute;
  bottom: 70px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 16px;
  background: var(--sl-primary);
  color: white;
  border-radius: var(--sl-radius-full);
  font-size: 0.75rem;
  cursor: pointer;
  box-shadow: var(--sl-shadow-md);
  z-index: 10;
}
.console-input-wrapper {
  position: relative;
  flex-shrink: 0;
}
.suggestions-popup {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  margin-bottom: 4px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 20;
  box-shadow: var(--sl-shadow-md);
}
.suggestion-item {
  padding: 6px 14px;
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  cursor: pointer;
  transition: background var(--sl-transition-fast);
}
.suggestion-item:hover,
.suggestion-item.active {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}
.suggestion-hint {
  padding: 4px 14px;
  font-size: 0.6875rem;
  color: var(--sl-text-tertiary);
  border-top: 1px solid var(--sl-border-light);
}
.console-input-bar {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
}
.input-prefix {
  color: var(--sl-primary);
  font-family: var(--sl-font-mono);
  font-weight: 700;
}
.console-input {
  flex: 1;
  background: transparent;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  padding: 6px 0;
  border: none;
  outline: none;
}
.console-input::placeholder {
  color: var(--sl-text-tertiary);
}
</style>
