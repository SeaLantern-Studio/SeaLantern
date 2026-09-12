<script setup lang="ts">
// keep-alive 缓存时 onUnmounted 不触发,改用 onActivated/onDeactivated 管理轮询
import { computed, onActivated, onDeactivated, onUnmounted, ref } from "vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { onTunnelEvent, tunnelApi, type OnlineTunnelEvent, type TunnelStatus } from "@api/tunnel";
import { settingsApi } from "@api/settings";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";
import { useToast } from "cmzya-modern-ui";
import { Copy, Eye, EyeOff, Github, Info, RefreshCw, X } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";

const DEFAULT_HOST_PORT = 25565;
const DEFAULT_JOIN_LOCAL_PORT = 30000;

const toast = useToast();
type PendingAction = "host" | "join" | "stop" | "generate-ticket";
const pendingAction = ref<PendingAction | null>(null);
const status = ref<TunnelStatus | null>(null);

const hostPort = ref(String(DEFAULT_HOST_PORT));
const hostPassword = ref("");
const hostRelayUrl = ref("");
const showHostPassword = ref(false);

const joinTicket = ref("");
const joinLocalPort = ref(String(DEFAULT_JOIN_LOCAL_PORT));
const joinPassword = ref("");
const showJoinPassword = ref(false);
const showInfoModal = ref(false);

const running = computed(() => status.value?.running ?? false);
const modeLabel = computed(() => {
  if (status.value?.mode === "host") return i18n.t("tunnel.mode_host");
  if (status.value?.mode === "join") return i18n.t("tunnel.mode_join");
  return "-";
});
const runningStatusText = computed(() =>
  running.value ? i18n.t("tunnel.yes") : i18n.t("tunnel.no"),
);
const runningStatusClass = computed<"running" | "stopped">(() =>
  running.value ? "running" : "stopped",
);
const hasTicket = computed(() => Boolean(status.value?.ticket));
const isIdle = computed(() => status.value?.phase === "idle");
const isBusy = computed(() => pendingAction.value !== null);
// TODO(backend): tunnel_copy_ticket / tunnel_generate_ticket / tunnel_regenerate_ticket
// 后端尚未实现，暂时禁用票据复制/生成入口，避免点击后调用报错。ticket 目前仅由
// host 成功后的 status.ticket 返回。
const canCopyTicket = computed(() => false);
const canGenerateTicket = computed(() => false);
const canStartHost = computed(() => isIdle.value && !isBusy.value);
const canStartJoin = computed(() => isIdle.value && !isBusy.value);
const canStopTunnel = computed(() => running.value && !isBusy.value);
const canEditHostForm = computed(() => isIdle.value && !isBusy.value);
const canEditJoinForm = computed(() => isIdle.value && !isBusy.value);
const hostPasswordInputType = computed(() => (showHostPassword.value ? "text" : "password"));
const joinPasswordInputType = computed(() => (showJoinPassword.value ? "text" : "password"));

const hostActionLoading = computed(() => pendingAction.value === "host");
const joinActionLoading = computed(() => pendingAction.value === "join");
const stopActionLoading = computed(() => pendingAction.value === "stop");
const generateTicketLoading = computed(() => pendingAction.value === "generate-ticket");

const canClearHostRelay = computed(() => canEditHostForm.value && hostRelayUrl.value.length > 0);
const canClearJoinTicket = computed(() => canEditJoinForm.value && joinTicket.value.length > 0);

const showConnections = computed(
  () => running.value && (status.value?.mode === "host" || status.value?.mode === "join"),
);

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: () => void;
  getAllPlainText: () => string;
}

const tunnelOutputRef = ref<ConsoleOutputExpose | null>(null);
const userScrolledUp = ref(false);
const consoleFontSize = ref(13);
const consoleFontFamily = ref("");
const consoleLetterSpacing = ref(0);
const maxLogLines = ref(5000);
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

function applyStatus(next: TunnelStatus) {
  status.value = next;
}

/** 隧道运行角色文案（复用 host_title / join_title） */
function formatTunnelMode(mode: "host" | "join"): string {
  return i18n.t(mode === "host" ? "tunnel.host_title" : "tunnel.join_title");
}

/** 把隧道事件映射为可读文本行（reason / message 为后端原文，直接展示不翻译） */
function formatTunnelEvent(event: OnlineTunnelEvent): string[] {
  switch (event.kind) {
    case "started": {
      const lines = [i18n.t("tunnel.ev_started", { mode: formatTunnelMode(event.mode) })];
      if (event.ticket) {
        lines.push(i18n.t("tunnel.ev_share_ticket", { ticket: event.ticket }));
      }
      return lines;
    }
    case "stopped":
      return [i18n.t("tunnel.ev_stopped", { mode: formatTunnelMode(event.mode) })];
    case "player_joined":
      return [i18n.t("tunnel.ev_player_joined", { remote_id: event.remote_id })];
    case "player_left":
      return [
        i18n.t("tunnel.ev_player_left", { remote_id: event.remote_id, reason: event.reason }),
      ];
    case "connected":
      return [i18n.t("tunnel.ev_connected")];
    case "disconnected":
      return [i18n.t("tunnel.ev_disconnected", { reason: event.reason })];
    case "path_changed":
      return [
        i18n.t("tunnel.ev_path_changed", {
          remote_id: event.remote_id,
          relay: i18n.t(event.is_relay ? "tunnel.route_relay" : "tunnel.route_direct"),
          rtt: event.rtt_ms,
        }),
      ];
    case "reconnecting":
      return [i18n.t("tunnel.ev_reconnecting", { attempt: event.attempt })];
    case "reconnected":
      return [i18n.t("tunnel.ev_reconnected")];
    case "authentication_failed":
      return [i18n.t("tunnel.ev_auth_failed", { remote_id: event.remote_id })];
    case "player_rejected":
      return [i18n.t("tunnel.ev_rejected", { remote_id: event.remote_id, reason: event.reason })];
    case "token_rotated":
      return [i18n.t("tunnel.ev_token_rotated")];
    case "error":
      return [i18n.t("tunnel.ev_error", { message: event.message })];
    case "provider_message":
      return [i18n.t("tunnel.ev_provider_message", { message: event.message })];
    default:
      // 后端新增事件类型时的兜底，避免静默变成 undefined
      return [i18n.t("tunnel.ev_unknown", { kind: (event as { kind: string }).kind })];
  }
}

/** 停用期间缓存的最大日志行数，避免长时间后台运行时无限增长。 */
const PENDING_LOG_LINES_MAX = 500;

/** 视图是否处于激活态；停用期间事件进入缓存而非直接写入输出区。 */
const isViewActive = ref(false);
const pendingLogLines: string[] = [];

function handleTunnelEvent(event: OnlineTunnelEvent) {
  const lines = formatTunnelEvent(event);
  if (isViewActive.value) {
    tunnelOutputRef.value?.appendLines(lines);
    return;
  }
  // 停用期间（keep-alive 缓存）仍保持订阅：先把日志行缓存起来，
  // 激活时回放，避免这段时间的事件永久丢失。
  pendingLogLines.push(...lines);
  if (pendingLogLines.length > PENDING_LOG_LINES_MAX) {
    pendingLogLines.splice(0, pendingLogLines.length - PENDING_LOG_LINES_MAX);
  }
}

/** 把停用期间缓存的日志行回放到输出区。 */
function flushPendingLogLines() {
  if (pendingLogLines.length === 0) return;
  tunnelOutputRef.value?.appendLines(pendingLogLines.splice(0));
}

// 事件订阅在组件生命周期内保持：keep-alive 停用期间不注销，
// 否则停用期事件会无来源可重放地永久丢失。
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

async function loadConsoleSettings() {
  try {
    const settings = await settingsApi.get();
    consoleFontSize.value = settings.console_font_size;
    consoleFontFamily.value = settings.console_font_family || "";
    consoleLetterSpacing.value = settings.console_letter_spacing || 0;
    maxLogLines.value = Math.max(100, settings.max_log_lines || 5000);
  } catch (e) {
    if (import.meta.env.DEV) console.warn("Failed to load console settings:", e);
  }
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
    const next = await tunnelApi.status();
    applyStatus(next);
  } catch (e) {
    if (!silent) toast.error(handleError(e));
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
    // 保证监听先于 host 命令注册,避免漏掉 started 事件
    await subscribeTunnelEvents();
    applyStatus(
      await tunnelApi.host({
        port: parsePort(hostPort.value, DEFAULT_HOST_PORT),
        password: hostPassword.value.trim() || undefined,
        relayUrl: hostRelayUrl.value.trim() || undefined,
      }),
    );
    toast.success(i18n.t("tunnel.host_started"));
  } catch (e) {
    toast.error(handleError(e));
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
  try {
    // 保证监听先于 join 命令注册,避免漏掉 started 事件
    await subscribeTunnelEvents();
    applyStatus(
      await tunnelApi.join({
        ticket: joinTicket.value,
        localPort: parsePort(joinLocalPort.value, DEFAULT_JOIN_LOCAL_PORT),
        password: joinPassword.value.trim() || undefined,
      }),
    );
    toast.success(i18n.t("tunnel.join_started"));
  } catch (e) {
    toast.error(handleError(e));
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
    toast.error(handleError(e));
  } finally {
    endAction("stop");
  }
}

async function copyTicket() {
  if (!canCopyTicket.value) return;
  try {
    const copied = await tunnelApi.copyTicket();
    if (copied) {
      toast.success(i18n.t("tunnel.ticket_copied"));
      applyStatus(await tunnelApi.status());
    } else {
      toast.error(i18n.t("tunnel.ticket_copy_failed"));
    }
  } catch (e) {
    toast.error(handleError(e));
  }
}

async function generateTicket() {
  if (!beginAction("generate-ticket")) return;
  try {
    applyStatus(await tunnelApi.generateTicket());
    toast.success(i18n.t("tunnel.ticket_generated"));
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    endAction("generate-ticket");
  }
}

async function regenerateTicket() {
  if (!beginAction("generate-ticket")) return;
  try {
    applyStatus(await tunnelApi.regenerateTicket());
    toast.success(i18n.t("tunnel.ticket_regenerated"));
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    endAction("generate-ticket");
  }
}

async function copyLogs() {
  const text = tunnelOutputRef.value?.getAllPlainText() || "";
  if (!text.trim()) return;

  const lineCount = text.split("\n").length;
  try {
    await navigator.clipboard.writeText(text);
    tunnelOutputRef.value?.appendLines([i18n.t("tunnel.log_copied", { count: lineCount })]);
  } catch {
    tunnelOutputRef.value?.appendLines([i18n.t("tunnel.log_copy_failed")]);
  }
}

function clearLogs() {
  tunnelOutputRef.value?.clear();
  userScrolledUp.value = false;
}

function clearHostRelay() {
  if (!canClearHostRelay.value) return;
  hostRelayUrl.value = "";
}

function clearJoinTicket() {
  if (!canClearJoinTicket.value) return;
  joinTicket.value = "";
}

function handleJoinTicketInput(value: string) {
  joinTicket.value = value;
}

onActivated(async () => {
  isViewActive.value = true;
  // 设置加载与状态拉取互不依赖,并行执行
  isPageVisible = true;
  await Promise.all([loadConsoleSettings(), refreshStatus()]);
  await subscribeTunnelEvents();
  flushPendingLogLines();
  startStatusPolling();
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onDeactivated(() => {
  isViewActive.value = false;
  stopStatusPolling();
  document.removeEventListener("visibilitychange", handleVisibilityChange);
  // 保持事件订阅：停用期间事件进入缓存，激活时回放，避免永久丢失。
});

// keep-alive 下组件常驻，真正卸载时才注销监听。
onUnmounted(() => {
  unsubscribeTunnelEvents();
});
</script>

<template>
  <div class="tunnel-view animate-stagger-in">
    <cmz-card :title="i18n.t('tunnel.status_title')" padding="md">
      <template #actions>
        <button
          class="ticket-icon-btn"
          :title="i18n.t('tunnel.info_title')"
          :aria-label="i18n.t('tunnel.info_title')"
          @click="showInfoModal = true"
        >
          <Info :size="16" />
        </button>
      </template>
      <div class="status-line">
        <span class="status-pill">
          <span class="status-pill-label">{{ i18n.t("tunnel.running") }}:</span>
          <cmz-badge
            dot
            :color="runningStatusClass === 'running' ? '#22c55e' : '#9ca3af'"
            :text="runningStatusText"
          />
        </span>
        <span class="status-pill">
          <span class="status-pill-label">{{ i18n.t("tunnel.mode") }}:</span>
          <span class="mode-text">{{ modeLabel }}</span>
        </span>
      </div>
      <div class="status-line status-line--ticket">
        <span class="ticket-label">{{ i18n.t("tunnel.ticket") }}:</span>
        <span class="ticket-scroll">
          <span class="ticket-text">{{ status?.ticket || i18n.t("tunnel.no_ticket") }}</span>
        </span>
        <span v-if="hasTicket" class="ticket-actions">
          <button
            class="ticket-icon-btn"
            :title="i18n.t('tunnel.copy_ticket')"
            :aria-label="i18n.t('tunnel.copy_ticket')"
            :disabled="!canCopyTicket"
            @click="copyTicket"
          >
            <Copy :size="16" />
          </button>
          <button
            class="ticket-icon-btn"
            :title="i18n.t('tunnel.regenerate_ticket')"
            :aria-label="i18n.t('tunnel.regenerate_ticket')"
            :disabled="!canGenerateTicket"
            @click="regenerateTicket"
          >
            <RefreshCw :size="16" />
          </button>
        </span>
        <cmz-button
          v-if="!hasTicket"
          variant="outline"
          size="sm"
          :disabled="!canGenerateTicket"
          :loading="generateTicketLoading"
          @click="generateTicket"
        >
          {{ i18n.t("tunnel.generate_ticket") }}
        </cmz-button>
      </div>
      <div v-if="running" class="status-actions">
        <cmz-button
          variant="solid"
          color="#ef4444"
          :disabled="!canStopTunnel"
          :loading="stopActionLoading"
          @click="stopTunnel"
        >
          {{ i18n.t("tunnel.stop") }}
        </cmz-button>
      </div>
    </cmz-card>

    <div class="tunnel-form-cards">
      <cmz-card :title="i18n.t('tunnel.host_title')" padding="md">
        <div class="form-grid">
          <cmz-input
            v-model="hostPort"
            :label="i18n.t('tunnel.host_port')"
            :disabled="!canEditHostForm"
          />
          <cmz-input
            v-model="hostPassword"
            :type="hostPasswordInputType"
            :label="i18n.t('tunnel.host_password')"
            :disabled="!canEditHostForm"
          >
            <template #suffix>
              <button
                type="button"
                class="ticket-icon-btn tunnel-input-icon-btn"
                :title="
                  showHostPassword ? i18n.t('tunnel.hide_password') : i18n.t('tunnel.show_password')
                "
                :aria-label="
                  showHostPassword ? i18n.t('tunnel.hide_password') : i18n.t('tunnel.show_password')
                "
                :disabled="!canEditHostForm"
                @click="showHostPassword = !showHostPassword"
              >
                <EyeOff v-if="showHostPassword" :size="14" />
                <Eye v-else :size="14" />
              </button>
            </template>
          </cmz-input>
          <cmz-input
            v-model="hostRelayUrl"
            :label="i18n.t('tunnel.host_relay_url')"
            :disabled="!canEditHostForm"
          >
            <template v-if="hostRelayUrl.length > 0" #suffix>
              <button
                type="button"
                class="ticket-icon-btn tunnel-input-icon-btn"
                :title="i18n.t('tunnel.clear_field')"
                :aria-label="i18n.t('tunnel.clear_field')"
                :disabled="!canClearHostRelay"
                @click="clearHostRelay"
              >
                <X :size="14" />
              </button>
            </template>
          </cmz-input>
        </div>
        <div class="card-actions">
          <cmz-button :disabled="!canStartHost" :loading="hostActionLoading" @click="startHost">
            {{ i18n.t("tunnel.start_host") }}
          </cmz-button>
        </div>
      </cmz-card>

      <cmz-card :title="i18n.t('tunnel.join_title')" variant="solid" padding="md">
        <div class="form-grid">
          <cmz-input
            :model-value="joinTicket"
            :label="i18n.t('tunnel.join_ticket')"
            :disabled="!canEditJoinForm"
            @update:model-value="handleJoinTicketInput"
          >
            <template v-if="joinTicket.length > 0" #suffix>
              <button
                type="button"
                class="ticket-icon-btn tunnel-input-icon-btn"
                :title="i18n.t('tunnel.clear_field')"
                :aria-label="i18n.t('tunnel.clear_field')"
                :disabled="!canClearJoinTicket"
                @click="clearJoinTicket"
              >
                <X :size="14" />
              </button>
            </template>
          </cmz-input>
          <cmz-input
            v-model="joinLocalPort"
            :label="i18n.t('tunnel.join_local_port')"
            :disabled="!canEditJoinForm"
          />
          <cmz-input
            v-model="joinPassword"
            :type="joinPasswordInputType"
            :label="i18n.t('tunnel.join_password')"
            :disabled="!canEditJoinForm"
          >
            <template #suffix>
              <button
                type="button"
                class="ticket-icon-btn tunnel-input-icon-btn"
                :title="
                  showJoinPassword ? i18n.t('tunnel.hide_password') : i18n.t('tunnel.show_password')
                "
                :aria-label="
                  showJoinPassword ? i18n.t('tunnel.hide_password') : i18n.t('tunnel.show_password')
                "
                :disabled="!canEditJoinForm"
                @click="showJoinPassword = !showJoinPassword"
              >
                <EyeOff v-if="showJoinPassword" :size="14" />
                <Eye v-else :size="14" />
              </button>
            </template>
          </cmz-input>
        </div>
        <div class="card-actions">
          <cmz-button
            variant="primary"
            :disabled="!canStartJoin"
            :loading="joinActionLoading"
            @click="startJoin"
          >
            {{ i18n.t("tunnel.start_join") }}
          </cmz-button>
        </div>
      </cmz-card>
    </div>

    <cmz-card
      v-if="showConnections"
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
              <th>{{ i18n.t("tunnel.table_tx") }}</th>
              <th>{{ i18n.t("tunnel.table_rx") }}</th>
              <th>{{ i18n.t("tunnel.table_alive") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in status?.connections" :key="item.remote_id">
              <td>{{ item.remote_id }}</td>
              <td>
                {{ item.is_relay ? i18n.t("tunnel.route_relay") : i18n.t("tunnel.route_direct") }}
              </td>
              <td>{{ item.rtt_ms }} ms</td>
              <td>{{ item.tx_bytes }}</td>
              <td>{{ item.rx_bytes }}</td>
              <td>{{ item.alive ? i18n.t("tunnel.yes") : i18n.t("tunnel.no") }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </cmz-card>

    <cmz-card
      class="tunnel-logs-card"
      :title="i18n.t('tunnel.logs_title')"
      variant="solid"
      padding="sm"
    >
      <template #actions>
        <div class="log-actions">
          <cmz-button variant="outline" size="sm" @click="copyLogs">
            {{ i18n.t("console.copy_log") }}
          </cmz-button>
          <cmz-button variant="ghost" size="sm" @click="clearLogs">
            {{ i18n.t("console.clear_log") }}
          </cmz-button>
        </div>
      </template>
      <div class="tunnel-log-console">
        <ConsoleOutput
          ref="tunnelOutputRef"
          :consoleFontSize="consoleFontSize"
          :consoleFontFamily="consoleFontFamily"
          :consoleLetterSpacing="consoleLetterSpacing"
          :maxLogLines="maxLogLines"
          :readonly="true"
          :userScrolledUp="userScrolledUp"
          @scroll="(value: boolean) => (userScrolledUp = value)"
          @scrollToBottom="
            userScrolledUp = false;
            tunnelOutputRef?.doScroll();
          "
        />
      </div>
    </cmz-card>

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
