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
const canStartHost = computed(() => isIdle.value && !isBusy.value);
const canStartJoin = computed(() => isIdle.value && !isBusy.value);
// 建链过程中也允许停止（即「取消连接」），否则用户无路可退。
// join 仍在收尾时按停止没有意义，避免重复触发。
const canStopTunnel = computed(
  () =>
    hasSession.value &&
    pendingAction.value !== "stop" &&
    !(stopRequested && pendingAction.value === "join"),
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
  if (pendingAction.value !== null) return false;
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
  try {
    applyStatus(
      await tunnelApi.host({
        port: parsePort(hostPort.value, DEFAULT_HOST_PORT),
        maxPlayers: parseMaxPlayers(hostMaxPlayers.value),
        relayUrl: hostRelayUrl.value.trim() || undefined,
        linkLifetime: hostLinkLifetime.value,
      }),
    );
    toast.success(i18n.t("tunnel.host_started"));
  } catch (e) {
    toast.error(tunnelError(e));
  } finally {
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
    toast.success(i18n.t("tunnel.join_started"));
  } catch (e) {
    // 用户主动取消时后端会让 join 以错误结束，不必提示。
    if (!stopRequested) toast.error(tunnelError(e));
  } finally {
    clearPendingPhase();
    endAction("join");
  }
}

async function stopTunnel() {
  if (!beginAction("stop")) return;
  stopRequested = true;
  try {
    applyStatus(await tunnelApi.stop());
    toast.success(i18n.t("tunnel.tunnel_stopped"));
  } catch (e) {
    toast.error(tunnelError(e));
  } finally {
    endAction("stop");
  }
}

async function copyText(value: string, successMessage: string) {
  if (!value) return;
  try {
    await navigator.clipboard.writeText(value);
    toast.success(successMessage);
  } catch (e) {
    toast.error(tunnelError(e));
  }
}

async function copyShareLink() {
  await copyText(shareLink.value, i18n.t("tunnel.ticket_copied"));
}

async function copyLocalAddress() {
  await copyText(localAddress.value, i18n.t("tunnel.address_copied"));
}

// 建立连接后立即补一次快照，不等下一个轮询周期。
watch(joined, (value) => {
  if (value && isPageVisible) void refreshStatus({ silent: true });
});

onActivated(async () => {
  isPageVisible = true;
  await refreshStatus();
  await subscribeTunnelEvents();
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
          <cmz-button :disabled="!canStartHost" :loading="hostActionLoading" @click="startHost">
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
          <cmz-button :disabled="!canStartJoin" :loading="joinActionLoading" @click="startJoin">
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
            {{ i18n.t("tunnel.copy_link") }}
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
              {{ i18n.t("tunnel.copy_address") }}
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
