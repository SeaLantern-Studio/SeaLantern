<script setup lang="ts">
// keep-alive 缓存时 onUnmounted 不触发,改用 onActivated/onDeactivated 管理轮询
import { computed, onActivated, onDeactivated, onUnmounted, ref, watch } from "vue";
import LatencyChart from "@components/views/tunnel/LatencyChart.vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  onTunnelEvent,
  tunnelApi,
  TUNNEL_LINK_LIFETIMES,
  type OnlineTunnelEvent,
  type OnlineTunnelPhase,
  type TunnelConnection,
  type TunnelLinkLifetime,
  type TunnelStatus,
} from "@api/tunnel";
import { setLiveRtt, getLiveRtt } from "@api/tunnelLatency";
import { settingsApi } from "@api/settings";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";
import { useToast } from "cmzya-modern-ui";
import { Github, Info } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";

const DEFAULT_HOST_PORT = 25565;
const DEFAULT_JOIN_LOCAL_PORT = 30000;

/** 快照轮询只兜底字节数、连接列表等低频指标；延迟趋势由 `path_changed` 事件驱动。 */
const STATUS_POLL_INTERVAL_MS = 5000;
/** 建链期间用更短的间隔跟进真实阶段。 */
const STARTING_POLL_INTERVAL_MS = 1000;

const toast = useToast();
type PendingAction = "host" | "join" | "stop";
const pendingAction = ref<PendingAction | null>(null);
/**
 * 仍在等待后端返回的命令数。
 *
 * 取消只让界面退出过渡态，旧命令还会在后台收尾（例如 host 仍在 bind），
 * 此间后端照样会拒绝新请求——所以期间要禁用建房/加入，而不是让用户点了才报 Busy。
 */
const commandsInFlight = ref(0);
const hasPendingCommand = computed(() => commandsInFlight.value > 0);
const status = ref<TunnelStatus | null>(null);
/** 用户在 `join` 命令返回前点了停止：后端会让 `join` 以错误结束，那不是故障。 */
let stopRequested = false;

const hostPort = ref(String(DEFAULT_HOST_PORT));
const hostMaxPlayers = ref("");
const hostRelayUrl = ref("");
const hostLinkLifetime = ref<TunnelLinkLifetime>("always");

const joinTicket = ref("");
const joinLocalPort = ref(String(DEFAULT_JOIN_LOCAL_PORT));
const showInfoModal = ref(false);

/** 联机偏好只回填一次，避免切页回来覆盖用户正在编辑的内容。 */
let tunnelPreferencesLoaded = false;

function isLinkLifetime(value: string | undefined): value is TunnelLinkLifetime {
  return value != null && (TUNNEL_LINK_LIFETIMES as readonly string[]).includes(value);
}

/** 从应用设置回填联机表单，省去每次重启重新填写。 */
async function loadTunnelPreferences() {
  if (tunnelPreferencesLoaded) return;
  try {
    const settings = await settingsApi.get();
    hostRelayUrl.value = settings.tunnel_relay_url ?? hostRelayUrl.value;
    if (isLinkLifetime(settings.tunnel_host_link_lifetime)) {
      hostLinkLifetime.value = settings.tunnel_host_link_lifetime;
    }
    if (settings.tunnel_join_port) {
      joinLocalPort.value = String(settings.tunnel_join_port);
    }
    if (settings.tunnel_host_max_players) {
      hostMaxPlayers.value = String(settings.tunnel_host_max_players);
    }
    joinTicket.value = settings.tunnel_join_uri ?? joinTicket.value;
    tunnelPreferencesLoaded = true;
  } catch {
    // 设置不可用时保留默认值，下次进入页面再试。
  }
}

/** 写入联机偏好；偏好落盘失败不应影响联机本身。 */
function saveTunnelPreferences(partial: {
  tunnel_relay_url?: string;
  tunnel_join_port?: number;
  tunnel_join_uri?: string;
  tunnel_host_link_lifetime?: TunnelLinkLifetime;
  tunnel_host_max_players?: number | null;
}) {
  void settingsApi.updatePartial(partial).catch(() => {});
}

/** 后端错误是 snake_case 的短标识，这里翻译成可操作的提示。 */
function tunnelError(error: unknown): string {
  const message = handleError(error);
  switch (message) {
    case "invalid_input":
      return i18n.t("tunnel.invalid_input");
    case "port_unavailable":
      return i18n.t("tunnel.port_unavailable");
    case "busy":
      return i18n.t("tunnel.busy");
    case "not_running":
      return i18n.t("tunnel.not_running");
    case "operation_failed":
      return i18n.t("tunnel.operation_failed");
    default:
      return message;
  }
}

/**
 * 本地乐观阶段。
 *
 * `tunnelApi.join/host` 要等隧道就绪才返回（join 最长 30s），期间后端拿不到中间
 * 状态；这里先本地进入过渡阶段，让页面立即切换并允许取消，命令返回后再以真实
 * 快照覆盖。
 */
const pendingPhase = ref<OnlineTunnelPhase | null>(null);

const running = computed(() => status.value?.running ?? false);
const currentPhase = computed(() => pendingPhase.value ?? status.value?.phase ?? "idle");
const isStarting = computed(() => currentPhase.value === "starting");
const joined = computed(() => status.value?.mode === "join" && (running.value || isStarting.value));
const hasSession = computed(() => isStarting.value || status.value?.mode != null);
const modeLabel = computed(() => {
  if (status.value?.mode === "host") return i18n.t("tunnel.mode_host");
  if (status.value?.mode === "join") return i18n.t("tunnel.mode_join");
  return "-";
});
const shareLink = computed(() =>
  status.value?.mode === "host" ? (status.value.ticket ?? "") : "",
);
const hasShareLink = computed(() => shareLink.value.length > 0);

/** 加入方要填进 Minecraft 多人游戏的地址。 */
const localAddress = computed(() => status.value?.localAddress ?? "");
const hasLocalAddress = computed(() => localAddress.value.length > 0);
const joinConnection = computed<TunnelConnection | null>(
  () => status.value?.connections?.[0] ?? null,
);
const joinRoute = computed(() => {
  const connection = joinConnection.value;
  if (!connection) return "";
  return connection.is_relay ? i18n.t("tunnel.route_relay") : i18n.t("tunnel.route_direct");
});
const joinLatency = computed(() => {
  const value = joinConnection.value?.rtt_ms;
  return value == null ? "--" : `${value} ms`;
});
const joinSent = computed(() => formatBytes(joinConnection.value?.tx_bytes ?? 0));
const joinReceived = computed(() => formatBytes(joinConnection.value?.rx_bytes ?? 0));
const joinStepTitle = computed(() =>
  currentPhase.value === "active" ? i18n.t("tunnel.join_ready") : i18n.t("tunnel.join_starting"),
);

const isIdle = computed(() => currentPhase.value === "idle");
const isBusy = computed(() => pendingAction.value !== null);
const canStartHost = computed(() => isIdle.value && !isBusy.value && !hasPendingCommand.value);
const canStartJoin = computed(() => isIdle.value && !isBusy.value && !hasPendingCommand.value);
// 建链过程中也允许停止（即「取消连接」），否则用户无路可退。
// 已经发过取消、命令仍在收尾时不再重复触发（host 与 join 同理）。
const canStopTunnel = computed(
  () => hasSession.value && pendingAction.value !== "stop" && !stopRequested,
);
const isCancellable = computed(() => isStarting.value);
const canEditHostForm = computed(() => isIdle.value && !isBusy.value);
const canEditJoinForm = computed(() => isIdle.value && !isBusy.value);

const hostActionLoading = computed(() => pendingAction.value === "host");
const joinActionLoading = computed(() => pendingAction.value === "join");
const stopActionLoading = computed(() => pendingAction.value === "stop");

// 房主展示玩家连接列表，加入方展示延迟趋势。
const showConnections = computed(() => running.value && status.value?.mode === "host");
const showLatencyChart = computed(() => joined.value);

const linkLifetimeOptions = computed(() =>
  TUNNEL_LINK_LIFETIMES.map((value) => ({
    value,
    label: i18n.t(`tunnel.lifetime_${value}`),
  })),
);

/** 分档色块文案，阈值与 LatencyChart 内的常量保持一致。 */
const latencyThresholds = computed(() => ({
  low: i18n.t("tunnel.latency_low", { threshold: 80 }),
  medium: i18n.t("tunnel.latency_medium", { low: 80, high: 180 }),
  high: i18n.t("tunnel.latency_high", { threshold: 180 }),
}));

let statusPollTimer: ReturnType<typeof setTimeout> | null = null;
// 页面隐藏时暂停轮询,避免后台无意义 IPC 开销
let isPageVisible = true;

function beginAction(action: PendingAction): boolean {
  // 旧命令仍在收尾时不接新动作：后端此刻仍是 Busy，接了只会弹错误。
  if (pendingAction.value !== null || hasPendingCommand.value) return false;
  pendingAction.value = action;
  return true;
}

function endAction(action: PendingAction) {
  if (pendingAction.value === action) {
    pendingAction.value = null;
  }
}

function parsePort(value: string, fallback: number): number {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed) || parsed <= 0 || parsed > 65535) return fallback;
  return parsed;
}

function validatePort(value: string, fieldName: string): string | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return i18n.t("tunnel.err_port_empty", { field: fieldName });
  }
  const parsed = Number.parseInt(trimmed, 10);
  if (Number.isNaN(parsed)) {
    return i18n.t("tunnel.err_port_invalid", { field: fieldName, value: trimmed });
  }
  if (parsed <= 0 || parsed > 65535) {
    return i18n.t("tunnel.err_port_out_of_range", { field: fieldName, min: 1, max: 65535 });
  }
  return null;
}

function parseMaxPlayers(value: string): number | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  const parsed = Number.parseInt(trimmed, 10);
  if (Number.isNaN(parsed) || parsed <= 0) return undefined;
  return parsed;
}

/** 流量按 1024 进制折算，保留一位小数直到两位数以上。 */
function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** index;
  return `${index === 0 || value >= 10 ? Math.round(value) : value.toFixed(1)} ${units[index]}`;
}

function applyStatus(next: TunnelStatus) {
  status.value = next;
  // 实时延迟以事件为准，首个快照只用来给趋势图播种。
  setLiveRtt(next.mode === "join" ? (next.connections?.[0]?.rtt_ms ?? null) : null);
}

/** 命令返回（成功或失败）后退出本地过渡阶段，改用真实快照。 */
function clearPendingPhase() {
  pendingPhase.value = null;
}

function handleTunnelEvent(event: OnlineTunnelEvent) {
  switch (event.kind) {
    case "token_rotated":
      void refreshStatus({ silent: true });
      break;
    case "path_changed":
      setLiveRtt(event.rtt_ms);
      break;
    default:
      break;
  }
}

// token 用于处理"listen 尚未 resolve 组件就卸载"的竞态,避免监听泄漏。
let tunnelEventUnlisten: UnlistenFn | null = null;
let tunnelEventToken = 0;

async function subscribeTunnelEvents() {
  if (tunnelEventUnlisten) return;
  const token = ++tunnelEventToken;
  const unlisten = await onTunnelEvent(handleTunnelEvent);
  if (token !== tunnelEventToken) {
    unlisten();
    return;
  }
  tunnelEventUnlisten = unlisten;
}

function unsubscribeTunnelEvents() {
  tunnelEventToken++;
  tunnelEventUnlisten?.();
  tunnelEventUnlisten = null;
}

function startStatusPolling() {
  stopStatusPolling();
  const schedule = () => {
    // 建链期间加速轮询，好让后端真实阶段尽快取代本地过渡状态。
    const interval = isStarting.value ? STARTING_POLL_INTERVAL_MS : STATUS_POLL_INTERVAL_MS;
    statusPollTimer = setTimeout(() => {
      if (isPageVisible) void refreshStatus({ silent: true });
      schedule();
    }, interval);
  };
  schedule();
}

function stopStatusPolling() {
  if (statusPollTimer) {
    clearTimeout(statusPollTimer);
    statusPollTimer = null;
  }
}

function handleVisibilityChange() {
  const visible = document.visibilityState === "visible";
  if (visible === isPageVisible) return;
  isPageVisible = visible;
  if (visible) {
    void refreshStatus({ silent: true });
    startStatusPolling();
  } else {
    stopStatusPolling();
  }
}

async function refreshStatus(options?: { silent?: boolean }) {
  const silent = options?.silent ?? false;
  try {
    applyStatus(await tunnelApi.status());
  } catch (e) {
    if (!silent) toast.error(tunnelError(e));
  }
}

async function startHost() {
  if (!beginAction("host")) return;
  const portError = validatePort(hostPort.value, i18n.t("tunnel.host_port"));
  if (portError) {
    toast.error(portError);
    endAction("host");
    return;
  }
  stopRequested = false;
  pendingPhase.value = "starting";
  commandsInFlight.value += 1;
  try {
    const relayUrl = hostRelayUrl.value.trim();
    const maxPlayers = parseMaxPlayers(hostMaxPlayers.value);
    applyStatus(
      await tunnelApi.host({
        port: parsePort(hostPort.value, DEFAULT_HOST_PORT),
        maxPlayers,
        relayUrl: relayUrl || undefined,
        linkLifetime: hostLinkLifetime.value,
      }),
    );
    saveTunnelPreferences({
      tunnel_relay_url: relayUrl,
      tunnel_host_link_lifetime: hostLinkLifetime.value,
      // 清空输入框时写入 null，才能真正取消人数上限。
      tunnel_host_max_players: maxPlayers ?? null,
    });
  } catch (e) {
    // 用户主动取消时后端会让 host 以错误结束，不必提示。
    if (!stopRequested) toast.error(tunnelError(e));
  } finally {
    commandsInFlight.value -= 1;
    clearPendingPhase();
    endAction("host");
  }
}

async function startJoin() {
  if (!beginAction("join")) return;
  const portError = validatePort(joinLocalPort.value, i18n.t("tunnel.join_local_port"));
  if (portError) {
    toast.error(portError);
    endAction("join");
    return;
  }
  if (!joinTicket.value.trim()) {
    toast.error(i18n.t("tunnel.err_ticket_empty"));
    endAction("join");
    return;
  }
  stopRequested = false;
  pendingPhase.value = "starting";
  commandsInFlight.value += 1;
  try {
    const snapshot = await tunnelApi.join({
      // 分享链接→Join URI 的归一化由后端完成,前端只做去空格。
      ticket: joinTicket.value.trim(),
      localPort: parsePort(joinLocalPort.value, DEFAULT_JOIN_LOCAL_PORT),
    });
    // 取消与就绪可能同时到达，以取消为准，避免又把隧道显示成已连接。
    if (stopRequested) {
      void refreshStatus({ silent: true });
      return;
    }
    applyStatus(snapshot);
    saveTunnelPreferences({
      tunnel_join_port: parsePort(joinLocalPort.value, DEFAULT_JOIN_LOCAL_PORT),
    });
  } catch (e) {
    // 用户主动取消时后端会让 join 以错误结束，不必提示。
    if (!stopRequested) toast.error(tunnelError(e));
  } finally {
    commandsInFlight.value -= 1;
    clearPendingPhase();
    endAction("join");
  }
}

async function stopTunnel() {
  // 停止/取消是打断动作：host/join 仍在进行时也必须能发起，
  // 所以不能走 beginAction——它会因为已经有 pending 动作而直接拒绝。
  if (!canStopTunnel.value) return;
  pendingAction.value = "stop";
  stopRequested = true;
  commandsInFlight.value += 1;
  try {
    applyStatus(await tunnelApi.stop());
    // 后端已确认空闲，立即退出过渡状态，不必等 host/join 命令收尾。
    clearPendingPhase();
  } catch (e) {
    toast.error(tunnelError(e));
  } finally {
    commandsInFlight.value -= 1;
    endAction("stop");
  }
}

/** 复制成功的反馈由按钮自身给出（短暂显示「已复制」），联机页只弹失败提示。 */
const copiedKind = ref<"link" | "address" | null>(null);
let copiedTimer: ReturnType<typeof setTimeout> | null = null;

async function copyText(value: string, kind: "link" | "address") {
  if (!value) return;
  try {
    await navigator.clipboard.writeText(value);
    copiedKind.value = kind;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copiedKind.value = null;
      copiedTimer = null;
    }, 1600);
  } catch (e) {
    toast.error(tunnelError(e));
  }
}

async function copyShareLink() {
  await copyText(shareLink.value, "link");
}

async function copyLocalAddress() {
  await copyText(localAddress.value, "address");
}

// 建立连接后立即补一次快照，不等下一个轮询周期。
watch(joined, (value) => {
  if (value && isPageVisible) void refreshStatus({ silent: true });
});

// 隧道终究起来了，说明这次取消没拦住，恢复停止按钮可用。
watch(running, (value) => {
  if (value) stopRequested = false;
});

// 只有真正连上才记住邀请链接：填错的票据不值得回填。
watch(
  () => running.value && status.value?.mode === "join",
  (connected) => {
    const ticket = joinTicket.value.trim();
    if (connected && ticket) saveTunnelPreferences({ tunnel_join_uri: ticket });
  },
);

onActivated(async () => {
  isPageVisible = true;
  await refreshStatus();
  await subscribeTunnelEvents();
  await loadTunnelPreferences();
  startStatusPolling();
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onDeactivated(() => {
  stopStatusPolling();
  document.removeEventListener("visibilitychange", handleVisibilityChange);
});

onUnmounted(() => {
  unsubscribeTunnelEvents();
  setLiveRtt(null);
  if (copiedTimer) clearTimeout(copiedTimer);
});
</script>

<template>
  <div class="tunnel-view online-workspace animate-stagger-in">
    <button
      v-if="isIdle"
      class="ticket-icon-btn setup-info"
      :title="i18n.t('tunnel.info_title')"
      :aria-label="i18n.t('tunnel.info_title')"
      @click="showInfoModal = true"
    >
      <Info :size="16" />
    </button>

    <section v-if="isIdle" class="setup-grid">
      <cmz-card class="mode-card" :title="i18n.t('tunnel.host_title')" padding="md">
        <div class="form-grid">
          <cmz-input
            v-model="hostPort"
            :label="i18n.t('tunnel.host_port')"
            :disabled="!canEditHostForm"
          />
          <cmz-input
            v-model="hostMaxPlayers"
            :label="i18n.t('tunnel.host_max_players')"
            :disabled="!canEditHostForm"
          />
          <cmz-input
            v-model="hostRelayUrl"
            :label="i18n.t('tunnel.host_relay_url')"
            :disabled="!canEditHostForm"
          />
          <cmz-select
            :model-value="hostLinkLifetime"
            :options="linkLifetimeOptions"
            :label="i18n.t('tunnel.host_link_lifetime')"
            :disabled="!canEditHostForm"
            @update:model-value="
              (value: string | number) => (hostLinkLifetime = value as TunnelLinkLifetime)
            "
          />
        </div>
        <div class="card-actions">
          <cmz-button
            :disabled="!canStartHost"
            :loading="hostActionLoading || hasPendingCommand"
            @click="startHost"
          >
            {{ i18n.t("tunnel.start_host") }}
          </cmz-button>
        </div>
      </cmz-card>

      <cmz-card class="mode-card" :title="i18n.t('tunnel.join_title')" variant="solid" padding="md">
        <div class="form-grid">
          <cmz-input
            v-model="joinTicket"
            :label="i18n.t('tunnel.share_link')"
            :disabled="!canEditJoinForm"
          />
          <cmz-input
            v-model="joinLocalPort"
            :label="i18n.t('tunnel.join_local_port')"
            :disabled="!canEditJoinForm"
          />
        </div>
        <div class="card-actions">
          <cmz-button
            :disabled="!canStartJoin"
            :loading="joinActionLoading || hasPendingCommand"
            @click="startJoin"
          >
            {{ i18n.t("tunnel.start_join") }}
          </cmz-button>
        </div>
      </cmz-card>
    </section>

    <section v-else class="active-stack">
      <cmz-card class="connection-panel" :title="modeLabel" padding="md">
        <div v-if="hasShareLink" class="share-block">
          <span>{{ i18n.t("tunnel.share_link") }}</span>
          <code>{{ shareLink }}</code>
          <cmz-button variant="outline" size="sm" @click="copyShareLink">
            {{ copiedKind === "link" ? i18n.t("tunnel.copied") : i18n.t("tunnel.copy_link") }}
          </cmz-button>
        </div>

        <div v-else class="join-step">
          <h4>{{ joinStepTitle }}</h4>
          <div class="address-block">
            <span>{{ i18n.t("tunnel.minecraft_address") }}</span>
            <code v-if="hasLocalAddress">{{ localAddress }}</code>
            <code v-else class="pending">{{ i18n.t("tunnel.allocating_port") }}</code>
            <cmz-button
              variant="outline"
              size="sm"
              :disabled="!hasLocalAddress"
              @click="copyLocalAddress"
            >
              {{
                copiedKind === "address" ? i18n.t("tunnel.copied") : i18n.t("tunnel.copy_address")
              }}
            </cmz-button>
          </div>
          <div class="join-metrics">
            <div>
              <span>{{ i18n.t("tunnel.join_route") }}</span>
              <strong>{{ joinRoute || i18n.t("tunnel.detecting") }}</strong>
            </div>
            <div>
              <span>{{ i18n.t("tunnel.table_rtt") }}</span>
              <strong>{{ joinLatency }}</strong>
            </div>
            <div>
              <span>{{ i18n.t("tunnel.sent") }}</span>
              <strong>{{ joinSent }}</strong>
            </div>
            <div>
              <span>{{ i18n.t("tunnel.received") }}</span>
              <strong>{{ joinReceived }}</strong>
            </div>
          </div>
          <p class="join-hint">
            {{
              isCancellable
                ? i18n.t("tunnel.cancel_hint")
                : currentPhase === "active"
                  ? i18n.t("tunnel.join_hint")
                  : i18n.t("tunnel.syncing")
            }}
          </p>
        </div>

        <div class="card-actions">
          <cmz-button
            color="#ef4444"
            :disabled="!canStopTunnel"
            :loading="stopActionLoading"
            @click="stopTunnel"
          >
            {{ isCancellable ? i18n.t("tunnel.cancel_join") : i18n.t("tunnel.stop") }}
          </cmz-button>
        </div>
      </cmz-card>

      <cmz-card
        v-if="showConnections"
        class="connections-card"
        :title="i18n.t('tunnel.connections_title')"
        variant="solid"
        padding="md"
      >
        <div v-if="!status?.connections?.length" class="empty-text">
          {{ i18n.t("tunnel.no_connections") }}
        </div>
        <div v-else class="table-wrap">
          <table class="conn-table">
            <thead>
              <tr>
                <th>{{ i18n.t("tunnel.table_remote") }}</th>
                <th>{{ i18n.t("tunnel.table_route") }}</th>
                <th>{{ i18n.t("tunnel.table_rtt") }}</th>
                <th>{{ i18n.t("tunnel.table_alive") }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in status.connections" :key="item.remote_id">
                <td>{{ item.remote_id }}</td>
                <td>
                  {{ item.is_relay ? i18n.t("tunnel.route_relay") : i18n.t("tunnel.route_direct") }}
                </td>
                <td>{{ item.rtt_ms }} ms</td>
                <td>{{ item.alive ? i18n.t("tunnel.yes") : i18n.t("tunnel.no") }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </cmz-card>

      <cmz-card
        v-if="showLatencyChart"
        :title="i18n.t('tunnel.latency_history')"
        variant="solid"
        padding="md"
      >
        <LatencyChart
          :value="joinConnection?.rtt_ms ?? null"
          :label="i18n.t('tunnel.latency_history')"
          :thresholds="latencyThresholds"
          :sample="getLiveRtt"
        />
      </cmz-card>
    </section>

    <cmz-modal
      :visible="showInfoModal"
      :title="i18n.t('tunnel.info_title')"
      width="420px"
      @close="showInfoModal = false"
    >
      <div class="tunnel-info-content">
        <p>{{ i18n.t("tunnel.info_desc") }}</p>
        <button class="tunnel-info-github" @click="openUrl('https://github.com/KercyDing/sculk')">
          <Github :size="16" />
          <span>{{ i18n.t("tunnel.info_github") }}</span>
        </button>
      </div>
    </cmz-modal>
  </div>
</template>
<style src="@styles/views/TunnelView.css" scoped></style>
