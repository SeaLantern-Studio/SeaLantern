import { computed, shallowRef, type ComponentPublicInstance } from "vue";
import { serverApi } from "@api/server";
import { useConsoleDisplaySettings } from "@composables/useConsoleDisplaySettings";
import { useConsoleLogStream } from "@composables/console/useConsoleLogStream";
import { useMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { useConsoleHistoryStore } from "@stores/consoleHistoryStore";
import { useNextInstanceWorkspaceContext } from "@src/composables/useNextInstanceWorkspace";
import type { ServerStatus } from "@type/common";
import { isActiveServerStatus } from "@utils/serverStatus";

export interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: () => void;
  getAllPlainText: () => string;
}

export type ConsoleOutputRefInstance = Element | ComponentPublicInstance | null;

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

function formatConsoleStatusLabel(status: string | undefined): string {
  if (status === "Running") return i18n.t("console.running");
  if (status === "Starting") return i18n.t("console.starting");
  if (status === "Stopping") return i18n.t("console.stopping");
  return i18n.t("console.stopped");
}

function canStartServer(status: ServerStatus | undefined): boolean {
  return status === "Stopped" || status === "Error" || status === undefined;
}

function canStopServer(status: ServerStatus | undefined): boolean {
  return status === "Running" || status === "Starting";
}

function canForceStopServer(status: ServerStatus | undefined): boolean {
  return status === "Running" || status === "Starting" || status === "Stopping";
}

export function useServerInstanceConsolePage() {
  const workspace = useNextInstanceWorkspaceContext();
  const message = useMessage();
  const commandHistoryStore = useConsoleHistoryStore();
  const consoleOutputRef = shallowRef<ConsoleOutputExpose | null>(null);
  const userScrolledUp = shallowRef(false);
  const sendingCommand = shallowRef(false);
  const lifecycleSubmitting = shallowRef(false);
  const forceStopDialogVisible = shallowRef(false);
  const forceStopSubmitting = shallowRef(false);

  const { consoleFontSize, consoleFontFamily, consoleLetterSpacing, maxLogLines } =
    useConsoleDisplaySettings();

  const stream = useConsoleLogStream({
    consoleOutputRef,
    userScrolledUp,
    maxLogLines,
    doScroll: () => consoleOutputRef.value?.doScroll(),
  });

  const serverId = computed(() => workspace.serverId.value);
  const runtimeStatus = computed(() => workspace.status.value?.status);
  const isActive = computed(() => {
    const status = runtimeStatus.value;
    return status ? isActiveServerStatus(status) : false;
  });
  const statusLabel = computed(() => formatConsoleStatusLabel(runtimeStatus.value));
  const commandHistory = computed(() => commandHistoryStore.getHistory(serverId.value));
  const canStart = computed(() => canStartServer(runtimeStatus.value));
  const canStop = computed(() => canStopServer(runtimeStatus.value));
  const canForceStop = computed(() => canForceStopServer(runtimeStatus.value));
  const primaryActionLabel = computed(() =>
    canStart.value ? i18n.t("home.start") : i18n.t("home.stop"),
  );
  const primaryActionVariant = computed(() => (canStart.value ? "primary" : "secondary"));
  const primaryActionDisabled = computed(() => {
    if (!serverId.value || lifecycleSubmitting.value) {
      return true;
    }

    if (runtimeStatus.value === "Stopping") {
      return true;
    }

    return !canStart.value && !canStop.value;
  });

  async function refresh(manual = false): Promise<void> {
    await workspace.refreshServerContext();
    if (!serverId.value) {
      return;
    }

    await stream.syncAndFocus(serverId.value);
    if (manual) {
      userScrolledUp.value = false;
    }
  }

  async function sendCommand(command: string): Promise<void> {
    if (!serverId.value || !isActive.value || sendingCommand.value) {
      return;
    }

    sendingCommand.value = true;
    message.clearAll();

    try {
      stream.appendCommandEcho(serverId.value, command);
      await serverApi.sendCommand(serverId.value, command);
      commandHistoryStore.pushCommand(serverId.value, command);
      message.showSuccess(i18n.t("common.message_command_sent"));
      userScrolledUp.value = false;
      consoleOutputRef.value?.doScroll();
    } catch (error) {
      message.showError(error instanceof Error ? error.message : String(error));
    } finally {
      sendingCommand.value = false;
    }
  }

  async function runPrimaryAction(): Promise<void> {
    if (!serverId.value || primaryActionDisabled.value) {
      return;
    }

    lifecycleSubmitting.value = true;
    message.clearAll();

    try {
      if (canStart.value) {
        await serverApi.start(serverId.value);
        message.showSuccess(i18n.t("common.message_server_started"));
      } else {
        await serverApi.stop(serverId.value);
        message.showSuccess(i18n.t("common.message_server_stopped"));
      }

      await refresh(false);
    } catch (error) {
      message.showError(error instanceof Error ? error.message : String(error));
    } finally {
      lifecycleSubmitting.value = false;
    }
  }

  function openForceStopDialog(): void {
    if (!canForceStop.value || forceStopSubmitting.value) {
      return;
    }

    forceStopDialogVisible.value = true;
  }

  function closeForceStopDialog(): void {
    if (forceStopSubmitting.value) {
      return;
    }

    forceStopDialogVisible.value = false;
  }

  async function confirmForceStop(): Promise<void> {
    if (!serverId.value || !canForceStop.value || forceStopSubmitting.value) {
      return;
    }

    forceStopSubmitting.value = true;
    message.clearAll();

    try {
      const { token } = await serverApi.prepareForceStop(serverId.value);
      await serverApi.forceStop(serverId.value, token);
      forceStopDialogVisible.value = false;
      message.showSuccess(i18n.t("console.force_stop_requested"));
      await refresh(false);
    } catch (error) {
      message.showError(error instanceof Error ? error.message : String(error));
    } finally {
      forceStopSubmitting.value = false;
    }
  }

  async function copyLogs(): Promise<void> {
    const text = consoleOutputRef.value?.getAllPlainText() ?? "";
    if (!text.trim()) {
      return;
    }

    try {
      await navigator.clipboard.writeText(text);
      message.showSuccess(i18n.t("console.copy_log"));
    } catch (error) {
      message.showError(error instanceof Error ? error.message : String(error));
    }
  }

  function clearLogs(): void {
    if (!serverId.value) {
      return;
    }

    stream.clearActiveLogs(serverId.value);
    userScrolledUp.value = false;
  }

  function scrollToBottom(): void {
    userScrolledUp.value = false;
    consoleOutputRef.value?.doScroll();
  }

  function setConsoleOutputRef(instance: ConsoleOutputRefInstance): void {
    consoleOutputRef.value = isConsoleOutputExpose(instance) ? instance : null;
  }

  return {
    errorMessage: message.error,
    successMessage: message.success,
    consoleFontSize,
    consoleFontFamily,
    consoleLetterSpacing,
    maxLogLines,
    serverId,
    userScrolledUp,
    commandHistory,
    sendingCommand,
    lifecycleSubmitting,
    isActive,
    canForceStop,
    statusLabel,
    primaryActionLabel,
    primaryActionVariant,
    primaryActionDisabled,
    forceStopDialogVisible,
    forceStopSubmitting,
    refresh,
    runPrimaryAction,
    openForceStopDialog,
    closeForceStopDialog,
    confirmForceStop,
    sendCommand,
    copyLogs,
    clearLogs,
    scrollToBottom,
    setConsoleOutputRef,
    activate: () => (serverId.value ? stream.activateLogStream(serverId.value) : Promise.resolve()),
    deactivate: stream.deactivateLogStream,
  };
}
