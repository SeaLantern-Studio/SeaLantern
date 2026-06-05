<script setup lang="ts">
import {
  ref,
  onMounted,
  onUnmounted,
  onActivated,
  onDeactivated,
  nextTick,
  computed,
  watch,
} from "vue";
import { ArrowDown } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import SLStatusIndicator from "@components/common/SLStatusIndicator.vue";
import ConsoleInput from "@components/console/ConsoleInput.vue";
import CommandModal from "@components/console/CommandModal.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { useConsoleDisplaySettings } from "@composables/useConsoleDisplaySettings";
import { useConsoleLogStream } from "@views/useConsoleLogStream";
import { useConsoleServerStats } from "@views/useConsoleServerStats";
import { useServerStore } from "@stores/serverStore";
import { useSettingsStore } from "@stores/settingsStore";
import { useRoute } from "vue-router";
import { serverApi } from "@api/server";
import { i18n } from "@language";
import { useLoading } from "@composables/useAsync";

const serverStore = useServerStore();
const settingsStore = useSettingsStore();
const route = useRoute();

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
const { consoleFontSize, consoleFontFamily, consoleLetterSpacing, maxLogLines } =
  useConsoleDisplaySettings();
const { loading: startLoading, start: startStartLoading, stop: stopStartLoading } = useLoading();
const { loading: stopLoading, start: startStopLoading, stop: stopStopLoading } = useLoading();
const {
  loading: forceStopLoading,
  start: startForceStopLoading,
  stop: stopForceStopLoading,
} = useLoading();
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
const {
  currentServer,
  statsSummaryItems,
  refreshServerStats,
  startStatsPolling,
  stopStatsPolling,
  resetServerStats,
} = useConsoleServerStats({
  serverId,
});
const serverStatusIndicator = computed<"running" | "starting" | "stopping" | "stopped">(() => {
  if (isRunning.value) return "running";
  if (isStarting.value) return "starting";
  if (isStopping.value) return "stopping";
  return "stopped";
});

const serverStatus = computed(() => serverStore.statuses[serverId.value]?.status || "Stopped");

const isRunning = computed(() => serverStatus.value === "Running");
const isStopping = computed(() => serverStatus.value === "Stopping");
const isStarting = computed(() => serverStatus.value === "Starting");
const {
  activateLogStream,
  deactivateLogStream,
  syncLogsOnce,
  syncAndFocus,
  clearActiveLogs,
  appendCommandEcho,
} = useConsoleLogStream({
  consoleOutputRef,
  userScrolledUp,
  maxLogLines,
  doScroll,
});

async function activateConsoleView() {
  const sid = serverId.value;
  if (sid) {
    await activateLogStream(sid);
  }

  if (serverId.value) {
    startStatsPolling();
  }
}

function deactivateConsoleView() {
  stopStatsPolling();
  deactivateLogStream();
}

onMounted(async () => {
  await settingsStore.ensureLoaded();

  await serverStore.refreshList();
  const routeId = typeof route.params.id === "string" ? route.params.id : "";
  if (routeId && serverStore.servers.some((server) => server.id === routeId)) {
    serverStore.setCurrentServer(routeId);
  } else if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (serverId.value) {
    await serverStore.refreshStatus(serverId.value);
    await syncLogsOnce(serverId.value);
  }
  await activateConsoleView();
  nextTick(() => doScroll());
});

onUnmounted(() => {
  deactivateConsoleView();
});

onActivated(async () => {
  await activateConsoleView();
});

onDeactivated(() => {
  deactivateConsoleView();
});

watch(
  () => route.params.id,
  (routeId) => {
    const normalizedId = typeof routeId === "string" ? routeId : "";
    if (!normalizedId) {
      return;
    }
    if (serverStore.servers.some((server) => server.id === normalizedId)) {
      serverStore.setCurrentServer(normalizedId);
    }
  },
);

watch(
  () => serverId.value,
  async (sid) => {
    resetServerStats();
    if (!sid) return;
    await serverStore.refreshStatus(sid);
    await syncAndFocus(sid);
    await activateLogStream(sid);
    startStatsPolling();
  },
);

async function sendCommand(cmd?: string) {
  const command = (cmd || commandInput.value).trim();
  const sid = serverId.value;
  if (!command || !sid) return;
  appendCommandEcho(sid, command);
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
  clearActiveLogs(serverId.value);
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
  <div class="console-view animate-fade-in-up">
    <div class="console-toolbar">
      <div class="toolbar-left">
        <span class="server-name-display">
          {{ currentServer?.name || i18n.t("console.no_server") }}
        </span>
        <SLStatusIndicator
          v-if="serverId"
          :status="serverStatusIndicator"
          :label="getStatusText()"
        />
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

      <div class="console-terminal-shell">
        <div class="console-terminal-section">
          <div class="console-terminal-toolbar">
            <div class="console-terminal-title">{{ i18n.t("console.title") }}</div>
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

          <div class="console-input-float">
            <ConsoleInput :consoleFontSize="consoleFontSize" @sendCommand="sendCommand" />
            <button
              v-if="userScrolledUp"
              type="button"
              class="scroll-to-bottom-btn"
              @click="
                userScrolledUp = false;
                doScroll();
              "
            >
              <ArrowDown :size="14" />
            </button>
          </div>
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
