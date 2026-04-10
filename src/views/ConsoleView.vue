<script setup lang="ts">
import { ref, onMounted, onUnmounted, onActivated, nextTick, computed, watch } from "vue";
import {
  ChevronDown,
  Cpu,
  HardDrive,
  MemoryStick,
  Menu,
  Gauge,
  Maximize2,
  Minimize2,
} from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import SLStatusIndicator from "@components/common/SLStatusIndicator.vue";
import SLSelect from "@components/common/SLSelect.vue";
import ConsoleInput from "@components/console/ConsoleInput.vue";
import CommandModal from "@components/console/CommandModal.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { useServerStore } from "@stores/serverStore";
import { serverApi } from "@api/server";
import { settingsApi } from "@api/settings";
import {
  serverSystemInfo,
  serverCpuUsage,
  serverMemUsage,
  serverDiskUsage,
  statsViewMode,
  serverStatsLoading,
  serverStatsError,
  serverCpuGaugeOption,
  serverMemGaugeOption,
  serverDiskGaugeOption,
  serverCpuLineOption,
  serverMemLineOption,
  fetchServerResourceUsage,
  resetStatsHistory,
  startThemeObserver,
  stopThemeObserver,
} from "@utils/statsUtils";
import { i18n } from "@language";
import { useLoading } from "@composables/useAsync";
import { SETTINGS_UPDATE_EVENT, type SettingsUpdateEvent } from "@stores/settingsStore";
import { formatBytes } from "@utils/serverUtils";
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
const consoleExpanded = ref(false);
const { loading: stopLoading, start: startStopLoading, stop: stopStopLoading } = useLoading();
const {
  loading: forceStopLoading,
  start: startForceStopLoading,
  stop: stopForceStopLoading,
} = useLoading();
let unlistenLogLine: UnlistenFn | null = null;
let statsTimer: ReturnType<typeof setInterval> | null = null;
const SERVER_STATS_POLL_INTERVAL_MS = 15000;
const forceStopConfirmVisible = ref(false);
const pendingForceStopServerId = ref("");
const pendingForceStopToken = ref("");

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
const currentServer = computed(
  () => serverStore.servers.find((server) => server.id === serverId.value) || null,
);
const serverProcessInfo = computed(() => serverSystemInfo.value);
const serverStatsUnavailable = computed(() => serverStatsError.value && !serverProcessInfo.value);
const noDataText = computed(() => {
  const text = i18n.t("home.no_data");
  return text === "home.no_data" ? i18n.t("common.unknown") : text;
});
const serverPidText = computed(() =>
  serverProcessInfo.value?.pid ? `PID ${serverProcessInfo.value.pid}` : noDataText.value,
);
const serverStatusIndicator = computed<"running" | "starting" | "stopping" | "stopped">(() => {
  if (isRunning.value) return "running";
  if (isStarting.value) return "starting";
  if (isStopping.value) return "stopping";
  return "stopped";
});
const serverOptions = computed(() =>
  serverStore.servers.map((server) => ({
    label: server.name,
    value: server.id,
  })),
);
const currentServerSelectValue = computed({
  get: () => serverStore.currentServerId ?? undefined,
  set: (value) => {
    if (!value) return;
    handleServerChange(String(value));
  },
});

const statsSummaryItems = computed(() => [
  {
    key: "cpu",
    icon: Cpu,
    label: i18n.t("home.cpu"),
    value: serverStatsUnavailable.value ? "--" : `${serverCpuUsage.value}%`,
    detail: serverPidText.value,
    tone: "primary",
  },
  {
    key: "memory",
    icon: MemoryStick,
    label: i18n.t("home.memory"),
    value: serverStatsUnavailable.value ? "--" : `${serverMemUsage.value}%`,
    detail:
      serverProcessInfo.value && serverProcessInfo.value.memory.total > 0
        ? `${formatBytes(serverProcessInfo.value.memory.used)} / ${formatBytes(serverProcessInfo.value.memory.total)}`
        : noDataText.value,
    tone: "success",
  },
  {
    key: "disk",
    icon: HardDrive,
    label: i18n.t("home.disk"),
    value: serverStatsUnavailable.value ? "--" : `${serverDiskUsage.value}%`,
    detail: serverProcessInfo.value
      ? `${formatBytes(serverProcessInfo.value.disk.used)} / ${formatBytes(serverProcessInfo.value.disk.total)}`
      : "--",
    tone: "warning",
  },
]);

const serverStatus = computed(() => serverStore.statuses[serverId.value]?.status || "Stopped");

const isRunning = computed(() => serverStatus.value === "Running");
const isStopping = computed(() => serverStatus.value === "Stopping");
const isStarting = computed(() => serverStatus.value === "Starting");

async function refreshServerStats() {
  const sid = serverId.value;
  if (!sid) {
    serverStatsLoading.value = false;
    return;
  }
  await fetchServerResourceUsage(sid);
}

function startStatsPolling() {
  stopStatsPolling();
  void refreshServerStats();
  statsTimer = setInterval(() => {
    void refreshServerStats();
  }, SERVER_STATS_POLL_INTERVAL_MS);
}

function stopStatsPolling() {
  if (statsTimer) {
    clearInterval(statsTimer);
    statsTimer = null;
  }
}

onMounted(async () => {
  await loadConsoleSettings();
  startThemeObserver();
  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);

  await serverStore.refreshList();
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (serverId.value) {
    await serverStore.refreshStatus(serverId.value);
    await syncLogsOnce(serverId.value);
    startStatsPolling();
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
  stopThemeObserver();
  stopStatsPolling();
  if (unlistenLogLine) {
    unlistenLogLine();
    unlistenLogLine = null;
  }
});

onActivated(async () => {
  await loadConsoleSettings();
  startThemeObserver();
  startStatsPolling();
});

watch(
  () => serverId.value,
  async (sid) => {
    resetStatsHistory();
    stopStatsPolling();
    if (!sid) return;
    await serverStore.refreshStatus(sid);
    await syncLogsOnce(sid);
    userScrolledUp.value = false;
    startStatsPolling();
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

function handleServerChange(value: string) {
  if (!value || value === serverStore.currentServerId) return;
  serverStore.setCurrentServer(value);
}

function toggleStatsViewMode() {
  statsViewMode.value = statsViewMode.value === "gauge" ? "detail" : "gauge";
}

function toggleConsoleExpanded() {
  consoleExpanded.value = !consoleExpanded.value;
  nextTick(() => doScroll());
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
    await refreshServerStats();
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
    await refreshServerStats();
  } catch (e) {
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
  } finally {
    stopStopLoading();
  }
}

async function handleForceStop(event?: Event) {
  event?.preventDefault();
  event?.stopPropagation();

  const sid = serverId.value;
  if (!sid || forceStopLoading.value) return;

  try {
    const preparation = await serverApi.prepareForceStop(sid);
    pendingForceStopServerId.value = sid;
    pendingForceStopToken.value = preparation.token;
    forceStopConfirmVisible.value = true;
  } catch (e) {
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
  }
}

function handleForceStopCancel() {
  forceStopConfirmVisible.value = false;
  pendingForceStopServerId.value = "";
  pendingForceStopToken.value = "";
}

async function confirmForceStop() {
  const sid = pendingForceStopServerId.value;
  const token = pendingForceStopToken.value;
  if (!sid || !token || forceStopLoading.value) {
    handleForceStopCancel();
    return;
  }

  startForceStopLoading();
  try {
    await serverApi.forceStop(sid, token);
    consoleOutputRef.value?.appendLines([
      "[Sea Lantern] " + i18n.t("console.force_stop_requested"),
    ]);
    await serverStore.refreshStatus(sid);
    await refreshServerStats();
  } catch (e) {
    consoleOutputRef.value?.appendLines(["[ERROR] " + String(e)]);
  } finally {
    stopForceStopLoading();
    handleForceStopCancel();
  }
}

function exportLogs() {
  const content = consoleOutputRef.value?.getAllPlainText() || "";
  if (!content) return;
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `console-${serverId.value || "server"}.log`;
  a.click();
  URL.revokeObjectURL(url);
}

function handleClearLogs() {
  consoleOutputRef.value?.clear();
}

function getStatusText() {
  if (isRunning.value) return i18n.t("home.running");
  if (isStarting.value) return i18n.t("home.starting");
  if (isStopping.value) return i18n.t("home.stopping");
  return i18n.t("home.stopped");
}

function saveCommand() {}
function deleteCommand() {}
</script>

<template>
  <div
    class="console-view animate-fade-in-up"
    :class="{ 'console-view--expanded': consoleExpanded }"
  >
    <div class="console-toolbar">
      <div class="toolbar-left toolbar-left--wide">
        <div class="server-picker-card">
          <div class="server-picker-label">{{ i18n.t("config.select_server") }}</div>
          <div class="server-picker-input">
            <SLSelect
              :modelValue="currentServerSelectValue"
              :options="serverOptions"
              :disabled="serverOptions.length === 0"
              style="width: 260px"
              @update:modelValue="handleServerChange(String($event))"
            />
            <ChevronDown class="server-picker-icon" :size="16" />
          </div>
        </div>
        <div class="server-meta-card">
          <div class="server-name-display">
            {{ currentServer?.name || i18n.t("console.no_server") }}
          </div>
          <div class="server-meta-line">
            <SLStatusIndicator
              v-if="serverId"
              :status="serverStatusIndicator"
              :label="getStatusText()"
            />
            <span v-if="currentServer?.path" class="server-path">{{ currentServer.path }}</span>
          </div>
        </div>
      </div>
      <div class="toolbar-right">
        <div class="action-group primary-actions">
          <SLButton
            v-if="isRunning || isStarting"
            type="button"
            variant="danger"
            size="sm"
            :loading="stopLoading"
            :disabled="isStopping || stopLoading || forceStopLoading"
            @click.stop.prevent="handleStop"
          >
            {{ isStarting ? i18n.t("home.stop") : i18n.t("home.stop") }}
          </SLButton>
          <SLButton
            v-if="isRunning || isStarting || isStopping"
            type="button"
            variant="secondary"
            size="sm"
            :loading="forceStopLoading"
            :disabled="forceStopLoading || stopLoading"
            @click.stop.prevent="handleForceStop"
          >
            {{ i18n.t("console.force_stop") }}
          </SLButton>
          <SLButton
            v-else
            type="button"
            variant="primary"
            size="sm"
            :loading="startLoading"
            :disabled="isStopping || startLoading || forceStopLoading"
            @click.stop.prevent="handleStart"
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
      <div
        class="console-overview-grid"
        :class="{ 'console-overview-grid--collapsed': consoleExpanded }"
      >
        <div class="console-stats-panel">
          <div class="console-stats-header">
            <div>
              <div class="console-stats-title">{{ i18n.t("home.system_resources") }}</div>
              <div class="console-stats-subtitle">
                {{ currentServer?.name || i18n.t("console.no_server") }}
              </div>
            </div>
            <button
              class="stats-view-toggle"
              type="button"
              :title="
                statsViewMode === 'gauge' ? i18n.t('home.detail_view') : i18n.t('home.gauge_view')
              "
              @click="toggleStatsViewMode"
            >
              <Menu v-if="statsViewMode === 'gauge'" :size="16" />
              <Gauge v-else :size="16" />
            </button>
          </div>

          <div class="console-stats-summary">
            <div
              v-for="item in statsSummaryItems"
              :key="item.key"
              class="stats-summary-card"
              :class="`stats-summary-card--${item.tone}`"
            >
              <component :is="item.icon" :size="16" class="stats-summary-icon" />
              <div class="stats-summary-content">
                <span class="stats-summary-label">{{ item.label }}</span>
                <strong class="stats-summary-value">{{ item.value }}</strong>
                <span class="stats-summary-detail">{{ item.detail }}</span>
              </div>
            </div>
          </div>

          <div v-if="serverStatsLoading" class="console-stats-loading">
            {{ i18n.t("common.loading") }}
          </div>
          <div v-else-if="serverStatsUnavailable" class="console-stats-loading">
            {{ noDataText }}
          </div>
          <div v-else-if="statsViewMode === 'gauge'" class="console-gauge-grid">
            <div class="console-gauge-item">
              <v-chart class="console-gauge-chart" :option="serverCpuGaugeOption" autoresize />
            </div>
            <div class="console-gauge-item">
              <v-chart class="console-gauge-chart" :option="serverMemGaugeOption" autoresize />
            </div>
            <div class="console-gauge-item">
              <v-chart class="console-gauge-chart" :option="serverDiskGaugeOption" autoresize />
            </div>
          </div>
          <div v-else class="console-line-grid">
            <div class="console-line-item">
              <div class="console-line-item-header">
                <span>{{ i18n.t("home.cpu") }}</span>
                <strong>{{ serverCpuUsage }}%</strong>
              </div>
              <v-chart class="console-line-chart" :option="serverCpuLineOption" autoresize />
            </div>
            <div class="console-line-item">
              <div class="console-line-item-header">
                <span>{{ i18n.t("home.memory") }}</span>
                <strong>{{ serverMemUsage }}%</strong>
              </div>
              <v-chart class="console-line-chart" :option="serverMemLineOption" autoresize />
            </div>
            <div class="console-line-item console-line-item--disk">
              <div class="console-line-item-header">
                <span>{{ i18n.t("home.disk") }}</span>
                <strong>{{ serverDiskUsage }}%</strong>
              </div>
              <div class="console-disk-bar-track">
                <div class="console-disk-bar-fill" :style="{ width: `${serverDiskUsage}%` }"></div>
              </div>
            </div>
          </div>
        </div>

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
      </div>

      <div
        class="console-terminal-shell"
        :class="{ 'console-terminal-shell--expanded': consoleExpanded }"
      >
        <div
          class="console-terminal-section"
          :class="{ 'console-terminal-section--expanded': consoleExpanded }"
        >
          <div class="console-terminal-toolbar">
            <div class="console-terminal-title">{{ i18n.t("console.title") }}</div>
            <button
              type="button"
              class="console-terminal-resize-btn"
              :title="consoleExpanded ? i18n.t('common.collapse') : i18n.t('common.expand')"
              @click="toggleConsoleExpanded"
            >
              <Minimize2 v-if="consoleExpanded" :size="16" />
              <Maximize2 v-else :size="16" />
            </button>
          </div>

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
        </div>

        <ConsoleInput :consoleFontSize="consoleFontSize" @sendCommand="sendCommand" />
      </div>

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

      <SLConfirmDialog
        :visible="forceStopConfirmVisible"
        :title="i18n.t('console.force_stop')"
        :message="i18n.t('console.force_stop_confirm')"
        :confirm-text="i18n.t('common.confirm')"
        :cancel-text="i18n.t('common.cancel')"
        confirm-variant="danger"
        :dangerous="true"
        :loading="forceStopLoading"
        @confirm="confirmForceStop"
        @cancel="handleForceStopCancel"
        @close="handleForceStopCancel"
      />
    </template>
  </div>
</template>
