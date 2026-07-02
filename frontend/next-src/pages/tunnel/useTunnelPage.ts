import { computed, onMounted, onUnmounted, shallowRef, type ComponentPublicInstance } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { tunnelApi, type TunnelStatus } from "@api/tunnel";
import { i18n } from "@language";
import type { WorkbenchFactItem } from "@next-src/components/workbench/WorkbenchFactGrid.vue";
import { useConsoleDisplaySettings } from "@composables/useConsoleDisplaySettings";
import { useGlobalMessage } from "@composables/useMessage";
import { useSettingsStore } from "@stores/settingsStore";
import { useTunnelStatusPolling } from "./useTunnelStatusPolling";

const DEFAULT_HOST_PORT = 25565;
const DEFAULT_JOIN_LOCAL_PORT = 30000;

type PendingAction = "host" | "join" | "stop" | "generate-ticket";

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: () => void;
  getAllPlainText: () => string;
}

type ConsoleOutputRefInstance = Element | ComponentPublicInstance | null;

function isConsoleOutputExpose(value: unknown): value is ConsoleOutputExpose {
  return (
    value !== null &&
    typeof value === "object" &&
    "doScroll" in value &&
    "appendLines" in value &&
    "clear" in value &&
    "getAllPlainText" in value
  );
}

function hasMatchingPrefix(lines: string[], prefixLength: number, previousLines: string[]): boolean {
  if (prefixLength === 0) {
    return false;
  }

  for (let index = 0; index < prefixLength; index += 1) {
    if (lines[index] !== previousLines[index]) {
      return false;
    }
  }

  return true;
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

async function openTunnelInfo(): Promise<void> {
  await openUrl("https://github.com/KercyDing/sculk");
}

export function useTunnelPage() {
  const globalMessage = useGlobalMessage();
  const settingsStore = useSettingsStore();
  const pendingAction = shallowRef<PendingAction | null>(null);
  const status = shallowRef<TunnelStatus | null>(null);
  const hostPort = shallowRef("");
  const hostPassword = shallowRef("");
  const hostRelayUrl = shallowRef("");
  const showHostPassword = shallowRef(false);
  const joinTicket = shallowRef("");
  const joinLocalPort = shallowRef("");
  const joinPassword = shallowRef("");
  const showJoinPassword = shallowRef(false);
  const joinTicketAutoFillEnabled = shallowRef(true);
  const showInfoModal = shallowRef(false);
  const tunnelOutputRef = shallowRef<ConsoleOutputExpose | null>(null);
  const userScrolledUp = shallowRef(false);
  const syncedLogCount = shallowRef(0);
  const syncedVisibleLogs = shallowRef<string[]>([]);
  const syncedFirstLogLine = shallowRef("");
  const syncedLastLogLine = shallowRef("");
  const logsDisplayClearedByUser = shallowRef(false);

  const { consoleFontSize, consoleFontFamily, consoleLetterSpacing, maxLogLines } =
    useConsoleDisplaySettings();

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
  const connectionRows = computed(() => status.value?.connections ?? []);
  const summaryFacts = computed<WorkbenchFactItem[]>(() => [
    {
      label: i18n.t("tunnel.running"),
      value: runningStatusText.value,
      tone: running.value ? "success" : "default",
    },
    {
      label: i18n.t("tunnel.mode"),
      value: modeLabel.value,
    },
    {
      label: i18n.t("tunnel.ticket"),
      value: status.value?.ticket || i18n.t("tunnel.no_ticket"),
      detail: status.value?.relay_url || undefined,
    },
    {
      label: i18n.t("tunnel.connections_title"),
      value: String(connectionRows.value.length),
    },
  ]);

  function syncLogOutput(logs: string[]) {
    const output = tunnelOutputRef.value;
    const visibleLogs = logs.filter((line) => !line.includes("host loop ended"));

    if (!output) {
      syncedLogCount.value = visibleLogs.length;
      syncedVisibleLogs.value = visibleLogs.slice();
      syncedFirstLogLine.value = visibleLogs[0] || "";
      syncedLastLogLine.value = visibleLogs[visibleLogs.length - 1] || "";
      return;
    }

    const missedInitialWrite =
      visibleLogs.length > 0 && !output.getAllPlainText().trim() && !logsDisplayClearedByUser.value;

    const canAppendOnly =
      syncedLogCount.value > 0 &&
      visibleLogs.length >= syncedLogCount.value &&
      visibleLogs[0] === syncedFirstLogLine.value &&
      visibleLogs[syncedLogCount.value - 1] === syncedLastLogLine.value &&
      hasMatchingPrefix(visibleLogs, syncedLogCount.value, syncedVisibleLogs.value);

    if (!canAppendOnly || missedInitialWrite) {
      logsDisplayClearedByUser.value = false;
      output.clear();
      if (visibleLogs.length > 0) {
        output.appendLines(visibleLogs);
      }
    } else {
      const delta = visibleLogs.slice(syncedLogCount.value);
      if (delta.length > 0) {
        output.appendLines(delta);
      }
    }

    syncedLogCount.value = visibleLogs.length;
    syncedVisibleLogs.value = visibleLogs.slice();
    syncedFirstLogLine.value = visibleLogs[0] || "";
    syncedLastLogLine.value = visibleLogs[visibleLogs.length - 1] || "";
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

  const { refreshStatus, startStatusPolling, stopStatusPolling } = useTunnelStatusPolling({
    applyStatus,
  });

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
    } catch (error) {
      globalMessage.error(String(error));
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
    } catch (error) {
      globalMessage.error(String(error));
    } finally {
      endAction("join");
    }
  }

  async function stopTunnel() {
    if (!beginAction("stop")) return;
    try {
      applyStatus(await tunnelApi.stop());
      globalMessage.success(i18n.t("tunnel.tunnel_stopped"));
    } catch (error) {
      globalMessage.error(String(error));
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
    } catch (error) {
      globalMessage.error(String(error));
    }
  }

  async function generateTicket() {
    if (!beginAction("generate-ticket")) return;
    try {
      applyStatus(await tunnelApi.generateTicket());
      globalMessage.success(i18n.t("tunnel.ticket_generated"));
    } catch (error) {
      globalMessage.error(String(error));
    } finally {
      endAction("generate-ticket");
    }
  }

  async function regenerateTicket() {
    if (!beginAction("generate-ticket")) return;
    try {
      applyStatus(await tunnelApi.regenerateTicket());
      globalMessage.success(i18n.t("tunnel.ticket_regenerated"));
    } catch (error) {
      globalMessage.error(String(error));
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

  function setTunnelOutputRef(instance: ConsoleOutputRefInstance) {
    tunnelOutputRef.value = isConsoleOutputExpose(instance) ? instance : null;
    if (tunnelOutputRef.value && status.value) {
      syncLogOutput(status.value.logs);
    }
  }

  onMounted(async () => {
    await settingsStore.ensureLoaded();
    await refreshStatus();
    startStatusPolling();
  });

  onUnmounted(() => {
    stopStatusPolling();
    syncedLogCount.value = 0;
    syncedVisibleLogs.value = [];
    syncedFirstLogLine.value = "";
    syncedLastLogLine.value = "";
    logsDisplayClearedByUser.value = false;
  });

  return {
    status,
    hostPort,
    hostPassword,
    hostRelayUrl,
    showHostPassword,
    joinTicket,
    joinLocalPort,
    joinPassword,
    showJoinPassword,
    showInfoModal,
    tunnelOutputRef,
    userScrolledUp,
    consoleFontSize,
    consoleFontFamily,
    consoleLetterSpacing,
    maxLogLines,
    running,
    modeLabel,
    runningStatusText,
    runningStatusClass,
    hasTicket,
    canCopyTicket,
    canGenerateTicket,
    canStartHost,
    canStartJoin,
    canStopTunnel,
    canEditHostForm,
    canEditJoinForm,
    hostPasswordInputType,
    joinPasswordInputType,
    hostActionLoading,
    joinActionLoading,
    stopActionLoading,
    generateTicketLoading,
    canClearHostRelay,
    canClearJoinTicket,
    showConnections,
    connectionRows,
    summaryFacts,
    startHost,
    startJoin,
    stopTunnel,
    copyTicket,
    generateTicket,
    regenerateTicket,
    copyLogs,
    clearLogs,
    clearHostRelay,
    clearJoinTicket,
    handleJoinTicketInput,
    setTunnelOutputRef,
    openTunnelInfo,
  };
}
