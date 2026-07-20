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
import { Cpu, HardDrive, MemoryStick } from "lucide-vue-next";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
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
  serverStatsLoading,
  serverStatsError,
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

// 命令补全 MD（Minecraft 原版常用命令）
const commandCompletionsMd = `
advancement
  grant
    <targets>
      everything
      only
        <advancement>
          <criterion>
      from
        <advancement>
      until
        <advancement>
      through
        <advancement>
  revoke
    <targets>
      everything
      only
        <advancement>
          <criterion>
      from
        <advancement>
      until
        <advancement>
      through
        <advancement>
attribute
  <target>
    <attribute>
      get
        <scale>
      base
        get
          <scale>
        set
          <value>
      modifier
        add
          <id> <name> <value> <operation>
        remove
          <id>
        get
          <id>
            <scale>
ban
  <targets>
    <reason>
ban-ip
  <target>
    <reason>
banlist
  ips
  players
bossbar
  add
    <id> <name>
  get
    <id>
      max
      players
      value
      visible
      name
      color
      style
  list
  remove
    <id>
  set
    <id>
      color
        <color>
      max
        <max>
      name
        <name>
      players
        <players>
      style
        <style>
      value
        <value>
      visible
        <visible>
clear
  <targets>
    <item>
      <max_count>
clone
  <begin> <end> <destination>
    replace
    masked
    filtered
      <block>
        force
        move
        normal
data
  get
    block <targetPos> <path> <scale>
    entity <target> <path> <scale>
    storage <source> <path> <scale>
  merge
    block <targetPos> <nbt>
    entity <target> <nbt>
    storage <target> <nbt>
  modify
    block <targetPos> <path> append <value>
    block <targetPos> <path> insert <index> <value>
    block <targetPos> <path> merge <value>
    block <targetPos> <path> prepend <value>
    block <targetPos> <path> set <value>
    block <targetPos> <path> remove
    entity <target> <path> append <value>
    entity <target> <path> insert <index> <value>
    entity <target> <path> merge <value>
    entity <target> <path> prepend <value>
    entity <target> <path> set <value>
    entity <target> <path> remove
    storage <target> <path> append <value>
    storage <target> <path> insert <index> <value>
    storage <target> <path> merge <value>
    storage <target> <path> prepend <value>
    storage <target> <path> set <value>
    storage <target> <path> remove
  remove
    block <targetPos> <path>
    entity <target> <path>
    storage <target> <path>
datapack
  list
    available
    enabled
  enable
    <name>
      after <existing>
      before <existing>
      last
      first
  disable
    <name>
debug
  start
  stop
  function
    <name>
  report
  clear
defaultgamemode
  survival
  creative
  adventure
  spectator
deop
  <players>
difficulty
  peaceful
  easy
  normal
  hard
effect
  clear
    <targets>
  give
    <targets> <effect>
      <seconds>
        <amplifier>
          <hideParticles>
enchant
  <targets> <enchantment>
    <level>
execute
  align <axes>
  anchored eyes
  anchored feet
  as <targets>
  at <targets>
  facing entity <entity>
  facing <pos>
  in <dimension>
  positioned <pos>
  rotated <rot>
  if block <pos> <block>
  if blocks <start> <end> <destination> all
  if blocks <start> <end> <destination> masked
  if entity <targets>
  if predicate <predicate>
  if score <target> <targetObjective> <=> <source> <sourceObjective>
  if score <target> <targetObjective> matches <range>
  unless block <pos> <block>
  unless blocks <start> <end> <destination> all
  unless blocks <start> <end> <destination> masked
  unless entity <targets>
  unless predicate <predicate>
  unless score <target> <targetObjective> <=> <source> <sourceObjective>
  unless score <target> <targetObjective> matches <range>
  run <command>
experience
  add <targets> <amount>
    points
    levels
  set <targets> <amount>
    points
    levels
  query <targets>
    points
    levels
fill
  <from> <to> <block>
    destroy
    hollow
    keep
    outline
    replace
      <filter>
fillbiome
  <from> <to> <biome>
forceload
  add
    <from>
      <to>
  remove
    <from>
      <to>
    all
  query
    <pos>
function
  <name>
gamemode
  survival
    <player>
  creative
    <player>
  adventure
    <player>
  spectator
    <player>
gamerule
  announceAdvancements
    true
    false
  commandBlockOutput
    true
    false
  disableElytraMovementCheck
    true
    false
  disableRaids
    true
    false
  doDaylightCycle
    true
    false
  doEntityDrops
    true
    false
  doFireTick
    true
    false
  doImmediateRespawn
    true
    false
  doInsomnia
    true
    false
  doLimitedCrafting
    true
    false
  doMobLoot
    true
    false
  doMobSpawning
    true
    false
  doPatrolSpawning
    true
    false
  doTileDrops
    true
    false
  doTraderSpawning
    true
    false
  doWeatherCycle
    true
    false
  doVinesSpread
    true
    false
  doWardenSpawning
    true
    false
  drowningDamage
    true
    false
  fallDamage
    true
    false
  fireDamage
    true
    false
  freezeDamage
    true
    false
  keepInventory
    true
    false
  logAdminCommands
    true
    false
  maxCommandChainLength
    <value>
  maxEntityCramming
    <value>
  mobGriefing
    true
    false
  naturalRegeneration
    true
    false
  playersSleepingPercentage
    <value>
  randomTickSpeed
    <value>
  reducedDebugInfo
    true
    false
  sendCommandFeedback
    true
    false
  showDeathMessages
    true
    false
  spawnRadius
    <value>
  spectatorsGenerateChunks
    true
    false
give
  <targets> <item>
    <count>
help
  <command>
item
  replace
    entity <targets> <slot> <item>
      <count>
    block <pos> <slot> <item>
      <count>
  modify
    entity <targets> <slot> <modifier>
    block <pos> <slot> <modifier>
jfr
  start
  stop
kick
  <targets>
    <reason>
kill
  <targets>
list
  uuids
locate
  biome <biome>
  structure <structure>
  poi <poi_type>
loot
  give <targets> <loot_table>
  fish <loot_table> <pos> <tool>
  kill <targets>
  mine <pos> <tool>
  insert <targetPos> <loot_table>
  replace entity <targets> <slot> <loot_table>
  replace block <pos> <slot> <loot_table>
me
  <action>
msg
  <target>
    <message>
particle
  <name> <pos>
    <delta_x> <delta_y> <delta_z> <speed> <count>
      force
      normal
      <viewers>
playsound
  <sound>
    master
    music
    record
    weather
    block
    hostile
    neutral
    player
    ambient
    voice
    <targets>
      <pos>
        <volume>
          <pitch>
            <minimum_volume>
publish
  <port>
recipe
  give
    <targets> <recipe>
  take
    <targets> <recipe>
reload
ride
  <target>
    mount
      <vehicle>
    dismount
    spawn_ride
      <entity>
say
  <message>
schedule
  function <function> <time>
    append
    replace
  clear
    <function>
scoreboard
  objectives
    list
    add <name> <criteria>
      <display_name>
    remove <name>
    setdisplay <slot> <objective>
    modify <name> displayname <name>
    modify <name> rendertype <type>
  players
    list
      <target>
    set <target> <objective> <score>
    add <target> <objective> <score>
    remove <target> <objective> <score>
    reset <target>
      <objective>
    enable <target> <trigger>
    operation <target> <targetObjective> <operation> <source> <sourceObjective>
  teams
    list
      <team>
    add <team>
      <display_name>
    remove <team>
    empty <team>
    join <team> <members>
    leave <members>
    modify <team>
      displayname <name>
      color <color>
      friendlyfire <allowed>
      seeFriendlyInvisibles <visible>
      nametagVisibility <visibility>
      deathMessageVisibility <visibility>
      collisionRule <rule>
      prefix <prefix>
      suffix <suffix>
      seeFriendlyInvisibles <enabled>
seed
setblock
  <pos> <block>
    destroy
    keep
    replace
setworldspawn
  <pos>
    <angle>
spawnpoint
  <targets>
    <pos>
      <angle>
spectate
  <target>
    <player>
spreadplayers
  <center> <spread_distance> <max_range> <max_teams>
    <teams>
      <respect_teams>
stop
stopsound
  <targets>
    *
    master
    music
    record
    weather
    block
    hostile
    neutral
    player
    ambient
    voice
    <sound>
summon
  <entity>
    <pos>
      <nbt>
tag
  <targets>
    add <name>
    remove <name>
    list
team
  list
    <team>
  add <team>
    <display_name>
  remove <team>
  empty <team>
  join <team> <members>
  leave <members>
  modify <team>
    displayname <name>
    color <color>
    friendlyfire <allowed>
    seeFriendlyInvisibles <visible>
    nametagVisibility <visibility>
    deathMessageVisibility <visibility>
    collisionRule <rule>
    prefix <prefix>
    suffix <suffix>
teammsg
  <message>
teleport
  <destination>
  <targets> <destination>
  <targets> <x> <y> <z>
    <yaw> <pitch>
tell
  <target>
    <message>
tellraw
  <targets>
    <message>
time
  set
    day
    night
    midnight
    noon
    <time>
  add
    <time>
  query
    daytime
    gametime
    day
title
  <targets>
    title <title>
    subtitle <subtitle>
    actionbar <actionbar>
    times <fadeIn> <stay> <fadeOut>
    clear
    reset
tm
  <message>
tp
  <destination>
  <targets> <destination>
  <targets> <x> <y> <z>
trigger
  <objective>
    <value>
    add <value>
weather
  clear
    <duration>
  rain
    <duration>
  thunder
    <duration>
w
  <target>
    <message>
whitelist
  on
  off
  list
  add <players>
  remove <players>
  reload
worldborder
  add <distance>
    <time>
  set <distance>
    <time>
  center <pos>
  damage
    buffer <distance>
    amount <damage_per_block>
  warning
    distance <distance>
    time <time>
  get
xp
  add <targets> <amount>
    points
    levels
  set <targets> <amount>
    points
    levels
  query <targets>
    points
    levels
save-all
  flush
save-on
save-off
stop
restart
list
tps
help
`;

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

const statusColor = computed(() => {
  if (isRunning.value) return "#22c55e";
  if (isStarting.value || isStopping.value) return "#f59e0b";
  return "#64748b";
});

const statsSummaryItems = computed(() => [
  {
    key: "cpu",
    icon: Cpu,
    label: i18n.t("home.cpu"),
    value: serverStatsUnavailable.value ? "--" : `${serverCpuUsage.value}%`,
    detail: "",
    tone: "primary",
  },
  {
    key: "memory",
    icon: MemoryStick,
    label: i18n.t("home.memory"),
    value:
      serverProcessInfo.value && currentServer.value
        ? `${formatBytes(serverProcessInfo.value.memory.used)} / ${currentServer.value.max_memory} MB`
        : "--",
    detail: "",
    tone: "success",
  },
  {
    key: "disk",
    icon: HardDrive,
    label: i18n.t("home.disk"),
    value: serverProcessInfo.value ? formatBytes(serverProcessInfo.value.disk.used) : "--",
    detail: "",
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
  await Promise.all([fetchServerResourceUsage(sid), serverStore.refreshStatus(sid)]);
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
    // 仅在服务器运行时启动资源轮询
    if (isRunning.value) startStatsPolling();
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
  // 仅在服务器运行时启动资源轮询
  if (isRunning.value) startStatsPolling();
  // 重新激活时重新加载当前服务器日志（可能在其它页面已启动服务器）
  const sid = serverId.value;
  if (sid) {
    await syncLogsOnce(sid);
    nextTick(() => doScroll());
  }
});

// keep-alive 缓存时 onUnmounted 不会触发,需在 onDeactivated 中暂停资源
// (unlistenLogLine 与 SETTINGS_UPDATE_EVENT 监听保留,切回时仍可用)
onDeactivated(() => {
  stopThemeObserver();
  stopStatsPolling();
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
    // 仅在服务器运行时启动资源轮询
    if (isRunning.value) startStatsPolling();
    nextTick(() => doScroll());
  },
);

// 服务器运行状态切换时启停轮询,停止后不再消耗 CPU/IO
watch(isRunning, (running) => {
  if (running) {
    startStatsPolling();
  } else {
    stopStatsPolling();
  }
});

async function syncLogsOnce(sid: string) {
  consoleOutputRef.value?.clear();
  try {
    const lines = await serverApi.getLogs(sid, 0, Math.max(1, maxLogLines.value));
    if (lines.length === 0) {
      consoleOutputRef.value?.appendLines(["[Sea Lantern] 该服务器尚无日志输出，请先启动服务器。"]);
    } else {
      consoleOutputRef.value?.appendLines(lines);
    }
  } catch (e) {
    console.warn("加载服务器日志失败:", e);
    consoleOutputRef.value?.appendLines([
      "[Sea Lantern] 无法加载此服务器的日志，该服务器可能尚未启动。",
    ]);
  }
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
  consoleOutputRef.value?.appendLines([`>>> ${command}`]);
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
    // 启动后重新拉取日志（可能在启动瞬间已有初始日志写入）
    await syncLogsOnce(sid);
    nextTick(() => doScroll());
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
  <div class="console-view animate-stagger-in">
    <div class="console-toolbar">
      <div class="toolbar-left">
        <span class="server-name-display">
          {{ currentServer?.name || i18n.t("console.no_server") }}
        </span>
        <cmz-badge v-if="serverId" dot :text="getStatusText()" :color="statusColor" />
      </div>
      <div class="toolbar-right">
        <div class="action-group primary-actions">
          <cmz-button
            v-if="isRunning || isStarting"
            type="button"
            variant="solid"
            color="#ef4444"
            size="sm"
            :loading="stopLoading"
            :disabled="isStopping || stopLoading || forceStopLoading"
            @click.stop.prevent="handleStop"
          >
            {{ isStarting ? i18n.t("home.stop") : i18n.t("home.stop") }}
          </cmz-button>
          <cmz-button
            v-if="isRunning || isStarting || isStopping"
            type="button"
            variant="outline"
            size="sm"
            :loading="forceStopLoading"
            :disabled="forceStopLoading || stopLoading"
            @click.stop.prevent="handleForceStop"
          >
            {{ i18n.t("console.force_stop") }}
          </cmz-button>
          <cmz-button
            v-else
            type="button"
            size="sm"
            :loading="startLoading"
            :disabled="isStopping || startLoading || forceStopLoading"
            @click.stop.prevent="handleStart"
          >
            {{ i18n.t("home.start") }}
          </cmz-button>
        </div>
        <div class="action-group secondary-actions">
          <cmz-button variant="outline" size="sm" @click="exportLogs">{{
            i18n.t("console.copy_log")
          }}</cmz-button>
          <cmz-button variant="ghost" size="sm" @click="handleClearLogs">{{
            i18n.t("console.clear_log")
          }}</cmz-button>
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
        <ConsoleOutput
          ref="consoleOutputRef"
          :consoleFontSize="consoleFontSize"
          :consoleFontFamily="consoleFontFamily"
          :consoleLetterSpacing="consoleLetterSpacing"
          :maxLogLines="maxLogLines"
          :history="commandHistory"
          :completionMd="commandCompletionsMd"
          @command="sendCommand"
        />

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
