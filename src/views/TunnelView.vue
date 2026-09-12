<script setup lang="ts">
// keep-alive 缓存时 onUnmounted 不触发,改用 onActivated/onDeactivated 管理轮询
import { computed, onActivated, onDeactivated, onUnmounted, ref } from "vue";
import LatencyChart from "@components/views/tunnel/LatencyChart.vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  onTunnelEvent,
  tunnelApi,
  TUNNEL_LINK_LIFETIMES,
  type OnlineTunnelEvent,
  type TunnelLinkLifetime,
  type TunnelStatus,
} from "@api/tunnel";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";
import { useToast } from "cmzya-modern-ui";
import { Github, Info } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";

const DEFAULT_HOST_PORT = 25565;
const DEFAULT_JOIN_LOCAL_PORT = 30000;

const toast = useToast();
type PendingAction = "host" | "join" | "stop";
const pendingAction = ref<PendingAction | null>(null);
const status = ref<TunnelStatus | null>(null);

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

const running = computed(() => status.value?.running ?? false);
const modeLabel = computed(() => {
  if (status.value?.mode === "host") return i18n.t("tunnel.mode_host");
  if (status.value?.mode === "join") return i18n.t("tunnel.mode_join");
  return "-";
});
const shareLink = computed(() =>
  status.value?.mode === "host" ? (status.value.ticket ?? "") : "",
);
const hasShareLink = computed(() => shareLink.value.length > 0);
const isIdle = computed(() => status.value?.phase === "idle");
const isBusy = computed(() => pendingAction.value !== null);
const canStartHost = computed(() => isIdle.value && !isBusy.value);
const canStartJoin = computed(() => isIdle.value && !isBusy.value);
const canStopTunnel = computed(() => running.value && !isBusy.value);
const canEditHostForm = computed(() => isIdle.value && !isBusy.value);
const canEditJoinForm = computed(() => isIdle.value && !isBusy.value);

const hostActionLoading = computed(() => pendingAction.value === "host");
const joinActionLoading = computed(() => pendingAction.value === "join");
const stopActionLoading = computed(() => pendingAction.value === "stop");

// 房主展示玩家连接列表，加入方展示延迟趋势。
const showConnections = computed(() => running.value && status.value?.mode === "host");
const showLatencyChart = computed(() => running.value && status.value?.mode === "join");
const latestRtt = computed(() => status.value?.connections?.[0]?.rtt_ms ?? null);

const linkLifetimeOptions = computed(() =>
  TUNNEL_LINK_LIFETIMES.map((value) => ({
    value,
    label: i18n.t(`tunnel.lifetime_${value}`),
  })),
);

let statusPollTimer: ReturnType<typeof setInterval> | null = null;
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

function applyStatus(next: TunnelStatus) {
  status.value = next;
}

// 令牌轮换会换掉分享链接，立即刷新而不是等下一次轮询。
function handleTunnelEvent(event: OnlineTunnelEvent) {
  if (event.kind === "token_rotated") {
    void refreshStatus({ silent: true });
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
  statusPollTimer = setInterval(() => {
    if (!isPageVisible) return;
    void refreshStatus({ silent: true });
  }, 5000);
}

function stopStatusPolling() {
  if (statusPollTimer) {
    clearInterval(statusPollTimer);
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
  try {
    applyStatus(
      await tunnelApi.join({
        // 分享链接→Join URI 的归一化由后端完成,前端只做去空格。
        ticket: joinTicket.value.trim(),
        localPort: parsePort(joinLocalPort.value, DEFAULT_JOIN_LOCAL_PORT),
      }),
    );
    toast.success(i18n.t("tunnel.join_started"));
  } catch (e) {
    toast.error(tunnelError(e));
  } finally {
    endAction("join");
  }
}

async function stopTunnel() {
  if (!beginAction("stop")) return;
  try {
    applyStatus(await tunnelApi.stop());
    toast.success(i18n.t("tunnel.tunnel_stopped"));
  } catch (e) {
    toast.error(tunnelError(e));
  } finally {
    endAction("stop");
  }
}

async function copyShareLink() {
  if (!shareLink.value) return;
  try {
    await navigator.clipboard.writeText(shareLink.value);
    toast.success(i18n.t("tunnel.ticket_copied"));
  } catch (e) {
    toast.error(tunnelError(e));
  }
}

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
        <div class="card-actions">
          <cmz-button
            color="#ef4444"
            :disabled="!canStopTunnel"
            :loading="stopActionLoading"
            @click="stopTunnel"
          >
            {{ i18n.t("tunnel.stop") }}
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
        <LatencyChart :value="latestRtt" :label="i18n.t('tunnel.latency_history')" />
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
