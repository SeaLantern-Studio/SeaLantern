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
import { useGlobalMessage } from "@composables/useMessage";
import { Copy, Eye, EyeOff, Github, Info, RefreshCw, X } from "lucide-vue-next";
import SLModal from "@components/common/SLModal.vue";
import { openUrl } from "@tauri-apps/plugin-opener";

const DEFAULT_HOST_PORT = 25565;
const DEFAULT_JOIN_LOCAL_PORT = 30000;

const globalMessage = useGlobalMessage();
type PendingAction = "host" | "join" | "stop" | "generate-ticket";
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
const joinTicketAutoFillEnabled = ref(true);
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
const isIdle = computed(() => !running.value);
const isBusy = computed(() => pendingAction.value !== null);
const canCopyTicket = computed(() => hasTicket.value && !isBusy.value);
const canGenerateTicket = computed(() => isIdle.value && !isBusy.value);
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
const syncedLogs = ref<string[]>([]);
const logsDisplayClearedByUser = ref(false);

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

function syncLogOutput(logs: string[]) {
  const output = tunnelOutputRef.value;
  if (!output) {
    syncedLogs.value = logs.slice();
    return;
  }

  const prev = syncedLogs.value;
  const canAppend = logs.length >= prev.length && prev.every((line, idx) => logs[idx] === line);

  const missedInitialWrite =
    logs.length > 0 && !output.getAllPlainText().trim() && !logsDisplayClearedByUser.value;

  if (!canAppend || missedInitialWrite) {
    logsDisplayClearedByUser.value = false;
    output.clear();
    if (logs.length > 0) {
      output.appendLines(logs.filter((l) => !l.includes("host loop ended")));
    }
  } else {
    const delta = logs.slice(prev.length);
    if (delta.length > 0) {
      output.appendLines(delta.filter((l) => !l.includes("host loop ended")));
    }
  }

  syncedLogs.value = logs.slice();
}

function applyStatus(next: TunnelStatus) {
  status.value = next;
  syncLogOutput(next.logs);
  if (!hostPort.value.trim()) hostPort.value = String(next.host_port);
  if (!joinLocalPort.value.trim()) joinLocalPort.value = String(next.join_port);
  if (!hostRelayUrl.value.trim() && next.relay_url) hostRelayUrl.value = next.relay_url;
  if (joinTicketAutoFillEnabled.value && !joinTicket.value.trim() && next.last_ticket) {
    joinTicket.value = next.last_ticket;
  }
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
  try {
    const next = await tunnelApi.status();
    applyStatus(next);
  } catch (e) {
    if (!silent) globalMessage.error(String(e));
  }
}

async function startHost() {
  if (!beginAction("host")) return;
  const portError = validatePort(hostPort.value, i18n.t("tunnel.host_port"));
  if (portError) {
    globalMessage.error(portError);
    endAction("host");
    return;
  }
  try {
    applyStatus(
      await tunnelApi.host({
        port: parsePort(hostPort.value, DEFAULT_HOST_PORT),
        password: hostPassword.value.trim() || undefined,
        relayUrl: hostRelayUrl.value.trim() || undefined,
      }),
    );
    globalMessage.success(i18n.t("tunnel.host_started"));
  } catch (e) {
    globalMessage.error(String(e));
  } finally {
    endAction("host");
  }
}

async function startJoin() {
  if (!beginAction("join")) return;
  const portError = validatePort(joinLocalPort.value, i18n.t("tunnel.join_local_port"));
  if (portError) {
    globalMessage.error(portError);
    endAction("join");
    return;
  }
  try {
    applyStatus(
      await tunnelApi.join({
        ticket: joinTicket.value,
        localPort: parsePort(joinLocalPort.value, DEFAULT_JOIN_LOCAL_PORT),
        password: joinPassword.value.trim() || undefined,
      }),
    );
    globalMessage.success(i18n.t("tunnel.join_started"));
  } catch (e) {
    globalMessage.error(String(e));
  } finally {
    endAction("join");
  }
}

async function stopTunnel() {
  if (!beginAction("stop")) return;
  try {
    applyStatus(await tunnelApi.stop());
    globalMessage.success(i18n.t("tunnel.tunnel_stopped"));
  } catch (e) {
    globalMessage.error(String(e));
  } finally {
    endAction("stop");
  }
}

async function copyTicket() {
  if (!canCopyTicket.value) return;
  try {
    const copied = await tunnelApi.copyTicket();
    if (copied) {
      globalMessage.success(i18n.t("tunnel.ticket_copied"));
      applyStatus(await tunnelApi.status());
    } else {
      globalMessage.error(i18n.t("tunnel.ticket_copy_failed"));
    }
  } catch (e) {
    globalMessage.error(String(e));
  }
}

async function generateTicket() {
  if (!beginAction("generate-ticket")) return;
  try {
    applyStatus(await tunnelApi.generateTicket());
    globalMessage.success(i18n.t("tunnel.ticket_generated"));
  } catch (e) {
    globalMessage.error(String(e));
  } finally {
    endAction("generate-ticket");
  }
}

async function regenerateTicket() {
  if (!beginAction("generate-ticket")) return;
  try {
    applyStatus(await tunnelApi.regenerateTicket());
    globalMessage.success(i18n.t("tunnel.ticket_regenerated"));
  } catch (e) {
    globalMessage.error(String(e));
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
  logsDisplayClearedByUser.value = true;
}

function clearHostRelay() {
  if (!canClearHostRelay.value) return;
  hostRelayUrl.value = "";
}

function clearJoinTicket() {
  if (!canClearJoinTicket.value) return;
  joinTicket.value = "";
  joinTicketAutoFillEnabled.value = false;
}

function handleJoinTicketInput(value: string) {
  joinTicket.value = value;
  joinTicketAutoFillEnabled.value = false;
}

onMounted(async () => {
  await loadConsoleSettings();
  await refreshStatus();
  startStatusPolling();
});

onUnmounted(() => {
  stopStatusPolling();
  syncedLogs.value = [];
  logsDisplayClearedByUser.value = false;
});
</script>

<template>
  <div class="tunnel-view animate-fade-in-up">
    <SLCard :title="i18n.t('tunnel.status_title')" variant="solid" padding="md">
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

    <SLCard
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
    </SLCard>

    <SLCard
      class="tunnel-logs-card"
      :title="i18n.t('tunnel.logs_title')"
      variant="solid"
      padding="sm"
    >
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

    <SLModal
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
    </SLModal>
  </div>
</template>

<style src="@styles/views/TunnelView.css" scoped></style>
