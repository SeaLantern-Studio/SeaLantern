<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from "vue";
import { useRoute } from "vue-router";
import SLButton from "../components/common/SLButton.vue";
import SLSelect from "../components/common/SLSelect.vue";
import SLInput from "../components/common/SLInput.vue";
import SLModal from "../components/common/SLModal.vue";
import { useServerStore } from "../stores/serverStore";
import { useConsoleStore } from "../stores/consoleStore";
import { serverApi } from "../api/server";
import { playerApi } from "../api/player";
import { settingsApi } from "../api/settings";
import { i18n } from "../locales";
import type { ServerCommand } from "../types/server";

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

watch(
  () => currentLogs.value.length,
  () => {
    if (!userScrolledUp.value) doScroll();
  },
);

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

function executeCustomCommand(cmd: ServerCommand) {
  sendCommand(cmd.command);
}
</script>

<template>
  <div class="console-view animate-fade-in-up">
    <div class="console-toolbar">
      <div class="toolbar-left">
        <div v-if="serverId" class="server-name-display">
          {{ serverStore.servers.find((s) => s.id === serverId)?.name || i18n.t("common.console") }}
        </div>
        <div v-else class="server-name-display">
          {{ i18n.t("home.no_servers") }}
        </div>
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
      <p class="text-body">{{ i18n.t("home.no_servers") }}</p>
    </div>

    <template v-else>
      <!-- 快捷指令和自定义指令部分 -->
      <div class="quick-commands">
        <!-- 快捷指令行 -->
        <div class="command-row">
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

        <!-- 自定义指令行 -->
        <div v-if="serverId" class="command-row custom-commands-row">
          <div class="custom-label">{{ i18n.t("console.custom") }}</div>
          <div class="custom-buttons">
            <div
              v-for="cmd in currentServerCommands"
              :key="cmd.id"
              class="custom-btn"
              @click="executeCustomCommand(cmd)"
              :title="cmd.command"
            >
              <span class="custom-btn-name">{{ cmd.name }}</span>
              <span class="custom-btn-edit" @click.stop="openEditCommandModal(cmd)"> ⚙️ </span>
            </div>
            <div
              class="custom-btn add-btn"
              @click="openAddCommandModal()"
              :title="i18n.t('console.add_custom_command')"
            >
              <span class="add-btn-plus">+</span>
            </div>
          </div>
        </div>
      </div>

      <div
        class="console-output"
        ref="logContainer"
        @scroll="handleScroll"
        :style="{ fontSize: consoleFontSize + 'px' }"
      >
        <div
          v-for="(line, i) in currentLogs"
          :key="i"
          class="log-line"
          :class="{
            'log-error':
              line.includes('[ERROR]') || line.includes('ERROR') || line.includes('[STDERR]'),
            'log-warn': line.includes('[WARN]') || line.includes('WARNING'),
            'log-command': line.startsWith('>'),
            'log-system': line.startsWith('[Sea Lantern]'),
          }"
        >
          <!-- 解析日志行，提取时间和等级 -->
          <template
            v-if="
              line.match(/^\[(\d{2}:\d{2}:\d{2})\] \[(.*?)\/(ERROR|INFO|WARN|DEBUG|FATAL)\]: (.*)$/)
            "
          >
            <span class="log-time">[{{ RegExp.$1 }}]</span>
            <span class="log-level" :class="'level-' + RegExp.$3.toLowerCase()"
              >[{{ RegExp.$2 }}/{{ RegExp.$3 }}]</span
            >
            <span class="log-content">{{ RegExp.$4 }}</span>
          </template>
          <!-- 对于不符合标准格式的日志行，直接显示 -->
          <template v-else>
            {{ line }}
          </template>
        </div>
        <div v-if="currentLogs.length === 0" class="log-empty">
          {{ i18n.t("console.waiting_for_output") }}
        </div>
      </div>

      <div
        v-if="userScrolledUp"
        class="scroll-btn"
        @click="
          userScrolledUp = false;
          doScroll();
        "
      >
        {{ i18n.t("console.back_to_bottom") }}
      </div>

      <div class="console-input-wrapper">
        <div v-if="showSuggestions && filteredSuggestions.length > 0" class="suggestions-popup">
          <div
            v-for="(sug, i) in filteredSuggestions"
            :key="sug"
            class="suggestion-item"
            :class="{ active: i === suggestionIndex }"
            @mousedown.prevent="
              commandInput = sug;
              showSuggestions = false;
            "
          >
            {{ sug }}
          </div>
          <div class="suggestion-hint">Tab 补全 / Up Down 选择</div>
        </div>
        <div class="console-input-bar">
          <span class="input-prefix">&gt;</span>
          <input
            class="console-input"
            v-model="commandInput"
            placeholder="输入命令... (Tab 补全)"
            @keydown="handleKeydown"
            :style="{ fontSize: consoleFontSize + 'px' }"
          />
          <SLButton variant="primary" size="sm" @click="sendCommand()">{{
            i18n.t("console.send_command")
          }}</SLButton>
        </div>
      </div>

      <!-- 自定义指令模态框 -->
      <SLModal
        :visible="showCommandModal"
        :title="commandModalTitle"
        :close-on-overlay="false"
        @close="showCommandModal = false"
      >
        <div class="command-modal-content">
          <div class="form-group">
            <label for="command-name">{{ i18n.t("console.command_name") }}</label>
            <SLInput
              id="command-name"
              v-model="commandName"
              :placeholder="i18n.t('console.enter_command_name')"
              :disabled="commandLoading"
            />
          </div>
          <div class="form-group">
            <label for="command-text">{{ i18n.t("console.command_content") }}</label>
            <SLInput
              id="command-text"
              v-model="commandText"
              :placeholder="i18n.t('console.enter_command_content')"
              :disabled="commandLoading"
            />
          </div>
        </div>
        <template #footer>
          <div class="modal-footer">
            <SLButton
              variant="secondary"
              @click="showCommandModal = false"
              :disabled="commandLoading"
            >
              {{ i18n.t("console.cancel") }}
            </SLButton>
            <SLButton
              v-if="editingCommand"
              variant="danger"
              @click="deleteCommand(editingCommand)"
              :disabled="commandLoading"
              :loading="commandLoading"
            >
              {{ i18n.t("console.delete") }}
            </SLButton>
            <SLButton
              variant="primary"
              @click="saveCommand"
              :disabled="!commandName.trim() || !commandText.trim() || commandLoading"
              :loading="commandLoading"
            >
              {{ editingCommand ? i18n.t("console.update") : i18n.t("console.add") }}
            </SLButton>
          </div>
        </template>
      </SLModal>
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
  flex-direction: column;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
}

.command-row {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
}

.custom-commands-row {
  margin-top: 2px;
}
.quick-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  white-space: nowrap;
  margin-top: 3px;
}
.quick-groups {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  align-items: center;
  flex: 1;
  min-width: 0;
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

/* 日志时间和等级样式 */
.log-time {
  color: var(--sl-text-tertiary);
  margin-right: 8px;
}

.log-level {
  margin-right: 8px;
  font-weight: 500;
}

.log-level.level-error {
  color: var(--sl-error);
}

.log-level.level-info {
  color: var(--sl-success);
}

.log-level.level-warn {
  color: var(--sl-warning);
}

.log-level.level-debug {
  color: var(--sl-info);
}

.log-level.level-fatal {
  color: var(--sl-error);
  font-weight: 700;
}

.log-content {
  color: var(--sl-text-primary);
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

/* 自定义指令样式 */
.custom-commands {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
  align-content: flex-start;
}

.custom-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  white-space: nowrap;
  margin-top: 3px;
}

.custom-buttons {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.custom-btn {
  position: relative;
  padding: 3px 10px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  background: var(--sl-bg-secondary);
  white-space: nowrap;
  transition: all var(--sl-transition-fast);
  min-width: 60px;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.custom-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.custom-btn-name {
  font-weight: 500;
  flex: 1;
}

.custom-btn-edit {
  margin-left: 6px;
  font-size: 0.85rem;
  opacity: 0;
  transform: scale(0.8);
  transition: all var(--sl-transition-fast);
  cursor: pointer;
}

.custom-btn:hover .custom-btn-edit {
  opacity: 1;
  transform: scale(1);
}

.custom-btn.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  font-size: 1.2rem;
  color: var(--sl-text-tertiary);
  border: 1px dashed var(--sl-border-light);
  background: transparent;
}

.custom-btn.add-btn:hover {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.add-btn-plus {
  line-height: 1;
}

/* 自定义指令模态框样式 */
.command-modal-content {
  padding: var(--sl-space-md);
  min-height: 120px;
}

.form-group {
  margin-bottom: var(--sl-space-md);
}

.form-group label {
  display: block;
  margin-bottom: var(--sl-space-xs);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
}
</style>
