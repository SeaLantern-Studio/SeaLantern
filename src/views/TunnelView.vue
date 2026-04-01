<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLStatusIndicator from "@components/common/SLStatusIndicator.vue";
import SLInput from "@components/common/SLInput.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { tunnelApi, type TunnelStatus } from "@api/tunnel";
import { settingsApi } from "@api/settings";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { Copy, Eye, EyeOff, RefreshCw, X } from "lucide-vue-next";

const { error, success, clearError, showError, showSuccess, clearAll } = useMessage();
type PendingAction = "host" | "join" | "stop" | "generate-ticket" | "copy-ticket";
const pendingAction = ref<PendingAction | null>(null);
const status = ref<TunnelStatus | null>(null);

const hostPort = ref("");
const hostPassword = ref("");
const hostRelayUrl = ref("");
const showHostPassword = ref(false);

const joinTicket = ref("");
const joinLocalPort = ref("");
const joinPassword = ref("");
const showJoinPassword = ref(false);

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
const isIdle = computed(() => !running.value);
const isBusy = computed(() => pendingAction.value !== null);
const canCopyTicket = computed(() => hasTicket.value && !isBusy.value);
const canGenerateTicket = computed(() => isIdle.value && !isBusy.value);
const canResetTicket = computed(() => hasTicket.value);
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
let syncedLogs: string[] = [];

function beginAction(action: PendingAction): boolean {
  if (pendingAction.value !== null) return false;
  pendingAction.value = action;
  clearAll();
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

function syncLogOutput(logs: string[]) {
  const output = tunnelOutputRef.value;
  if (!output) {
    syncedLogs = logs.slice();
    return;
  }

  const canAppend =
    logs.length >= syncedLogs.length && syncedLogs.every((line, idx) => logs[idx] === line);

  if (!canAppend) {
    output.clear();
    if (logs.length > 0) output.appendLines(logs);
  } else {
    const delta = logs.slice(syncedLogs.length);
    if (delta.length > 0) output.appendLines(delta);
  }

  syncedLogs = logs.slice();
}

function applyStatus(next: TunnelStatus) {
  status.value = next;
  syncLogOutput(next.logs);
  if (!hostPort.value.trim()) hostPort.value = String(next.host_port);
  if (!joinLocalPort.value.trim()) joinLocalPort.value = String(next.join_port);
  if (!hostRelayUrl.value.trim() && next.relay_url) hostRelayUrl.value = next.relay_url;
  if (!joinTicket.value.trim() && next.last_ticket) joinTicket.value = next.last_ticket;
}

async function loadConsoleSettings() {
  try {
    const settings = await settingsApi.get();
    consoleFontSize.value = settings.console_font_size;
    consoleFontFamily.value = settings.console_font_family || "";
    consoleLetterSpacing.value = settings.console_letter_spacing || 0;
    maxLogLines.value = Math.max(100, settings.max_log_lines || 5000);
  } catch {
    // keep defaults
  }
}

function startStatusPolling() {
  stopStatusPolling();
  statusPollTimer = setInterval(() => {
    void refreshStatus({ silent: true });
  }, 2000);
}

function stopStatusPolling() {
  if (statusPollTimer) {
    clearInterval(statusPollTimer);
    statusPollTimer = null;
  }
}

async function refreshStatus(options?: { silent?: boolean }) {
  const silent = options?.silent ?? false;
  if (!silent) clearAll();
  try {
    const next = await tunnelApi.status();
    applyStatus(next);
  } catch (e) {
    if (!silent) showError(String(e));
  }
}

async function startHost() {
  if (!beginAction("host")) return;
  try {
    applyStatus(
      await tunnelApi.host({
        port: parsePort(hostPort.value, 25565),
        password: hostPassword.value.trim() || undefined,
        relayUrl: hostRelayUrl.value.trim() || undefined,
      }),
    );
  } catch (e) {
    showError(String(e));
  } finally {
    endAction("host");
  }
}

async function startJoin() {
  if (!beginAction("join")) return;
  try {
    applyStatus(
      await tunnelApi.join({
        ticket: joinTicket.value,
        localPort: parsePort(joinLocalPort.value, 30000),
        password: joinPassword.value.trim() || undefined,
      }),
    );
  } catch (e) {
    showError(String(e));
  } finally {
    endAction("join");
  }
}

async function stopTunnel() {
  if (!beginAction("stop")) return;
  try {
    applyStatus(await tunnelApi.stop());
  } catch (e) {
    showError(String(e));
  } finally {
    endAction("stop");
  }
}

async function copyTicket() {
  if (!beginAction("copy-ticket")) return;
  try {
    const copied = await tunnelApi.copyTicket();
    if (copied) {
      showSuccess(i18n.t("tunnel.ticket_copied"));
      applyStatus(await tunnelApi.status());
    } else {
      showError(i18n.t("tunnel.ticket_copy_failed"));
    }
  } catch (e) {
    showError(String(e));
  } finally {
    endAction("copy-ticket");
  }
}

async function generateTicket() {
  if (!beginAction("generate-ticket")) return;
  try {
    applyStatus(await tunnelApi.generateTicket());
    showSuccess(i18n.t("tunnel.ticket_generated"));
  } catch (e) {
    showError(String(e));
  } finally {
    endAction("generate-ticket");
  }
}

async function copyLogs() {
  const text = tunnelOutputRef.value?.getAllPlainText() || "";
  if (!text.trim()) return;

  try {
    await navigator.clipboard.writeText(text);
    showSuccess(i18n.t("common.copied"));
  } catch (e) {
    showError(String(e));
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

onMounted(async () => {
  await loadConsoleSettings();
  await refreshStatus();
  startStatusPolling();
});

onUnmounted(() => {
  stopStatusPolling();
});
</script>

<template>
  <div class="tunnel-view animate-fade-in-up">
    <div v-if="error" class="msg-banner error-banner">
      <span>{{ error }}</span>
      <button @click="clearError">x</button>
    </div>
    <div v-if="success" class="msg-banner success-banner">
      <span>{{ success }}</span>
      <button @click="clearAll">x</button>
    </div>

    <SLCard :title="i18n.t('tunnel.status_title')" variant="solid" padding="md">
      <div class="status-line">
        <span class="status-pill">
          <span class="status-pill-label">{{ i18n.t("tunnel.running") }}:</span>
          <SLStatusIndicator :status="runningStatusClass" :label="runningStatusText" />
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
        <span v-if="canCopyTicket || canResetTicket" class="ticket-actions">
          <button
            v-if="canCopyTicket"
            class="ticket-icon-btn"
            :title="i18n.t('tunnel.copy_ticket')"
            :aria-label="i18n.t('tunnel.copy_ticket')"
            :disabled="!canCopyTicket"
            @click="copyTicket"
          >
            <Copy :size="16" />
          </button>
          <button
            v-if="canResetTicket"
            class="ticket-icon-btn"
            :title="i18n.t('tunnel.generate_ticket')"
            :aria-label="i18n.t('tunnel.generate_ticket')"
            :disabled="!canGenerateTicket"
            @click="generateTicket"
          >
            <RefreshCw :size="16" />
          </button>
        </span>
        <SLButton
          v-if="!hasTicket"
          variant="secondary"
          size="sm"
          :disabled="!canGenerateTicket"
          :loading="generateTicketLoading"
          @click="generateTicket"
        >
          {{ i18n.t("tunnel.generate_ticket") }}
        </SLButton>
      </div>
      <div v-if="running" class="status-actions">
        <SLButton
          variant="danger"
          :disabled="!canStopTunnel"
          :loading="stopActionLoading"
          @click="stopTunnel"
        >
          {{ i18n.t("tunnel.stop") }}
        </SLButton>
      </div>
    </SLCard>

    <div class="tunnel-form-cards">
      <SLCard :title="i18n.t('tunnel.host_title')" variant="solid" padding="md">
        <div class="form-grid">
          <SLInput
            v-model="hostPort"
            :label="i18n.t('tunnel.host_port')"
            :disabled="!canEditHostForm"
          />
          <SLInput
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
          </SLInput>
          <SLInput
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
          </SLInput>
        </div>
        <div class="card-actions">
          <SLButton
            variant="primary"
            :disabled="!canStartHost"
            :loading="hostActionLoading"
            @click="startHost"
          >
            {{ i18n.t("tunnel.start_host") }}
          </SLButton>
        </div>
      </SLCard>

      <SLCard :title="i18n.t('tunnel.join_title')" variant="solid" padding="md">
        <div class="form-grid">
          <SLInput
            v-model="joinTicket"
            :label="i18n.t('tunnel.join_ticket')"
            :disabled="!canEditJoinForm"
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
          </SLInput>
          <SLInput
            v-model="joinLocalPort"
            :label="i18n.t('tunnel.join_local_port')"
            :disabled="!canEditJoinForm"
          />
          <SLInput
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
          </SLInput>
        </div>
        <div class="card-actions">
          <SLButton
            variant="primary"
            :disabled="!canStartJoin"
            :loading="joinActionLoading"
            @click="startJoin"
          >
            {{ i18n.t("tunnel.start_join") }}
          </SLButton>
        </div>
      </SLCard>
    </div>

    <SLCard :title="i18n.t('tunnel.connections_title')" variant="solid" padding="md">
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
    </SLCard>

    <SLCard :title="i18n.t('tunnel.logs_title')" variant="solid" padding="md">
      <template #actions>
        <div class="log-actions">
          <SLButton variant="secondary" size="sm" @click="copyLogs">
            {{ i18n.t("console.copy_log") }}
          </SLButton>
          <SLButton variant="ghost" size="sm" @click="clearLogs">
            {{ i18n.t("console.clear_log") }}
          </SLButton>
        </div>
      </template>
      <div class="tunnel-log-console">
        <ConsoleOutput
          ref="tunnelOutputRef"
          :consoleFontSize="consoleFontSize"
          :consoleFontFamily="consoleFontFamily"
          :consoleLetterSpacing="consoleLetterSpacing"
          :maxLogLines="maxLogLines"
          :userScrolledUp="userScrolledUp"
          @scroll="(value) => (userScrolledUp = value)"
          @scrollToBottom="
            userScrolledUp = false;
            tunnelOutputRef?.doScroll();
          "
        />
      </div>
    </SLCard>
  </div>
</template>

<style src="@styles/views/TunnelView.css" scoped></style>
