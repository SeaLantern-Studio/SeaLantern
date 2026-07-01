import { computed, onMounted, ref } from "vue";
import { uiShellApi, type UiShellInfo, type UiShellStatus } from "@api/uiShell";
import { systemApi } from "@api/system";
import { isBrowserEnv } from "@api/tauri";
import type { UiShellId } from "@api/settings";
import { i18n } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";

const DEV_RESTART_MANUAL_REQUIRED_CODE = "plugins.ui_shell.dev_restart_manual_required";

interface UiShellCardViewModel {
  id: UiShellId;
  name: string;
  description: string;
  sourceLabel: string;
  running: boolean;
  selected: boolean;
  pendingRestart: boolean;
  available: boolean;
  disabled: boolean;
  actionLabel: string;
  statusBadges: Array<{
    text: string;
    variant: "primary" | "success" | "warning" | "neutral" | "info";
  }>;
}

const UI_SHELL_ORDER: UiShellId[] = ["classic", "next"];

function createFallbackShell(id: UiShellId): UiShellInfo {
  return {
    id,
    name: i18n.t(`plugins.ui_shell.shells.${id}.name`),
    description: i18n.t(`plugins.ui_shell.shells.${id}.description`),
    source: "builtin",
    builtin: true,
    available: id === "classic",
  };
}

export function useUiShellPanel() {
  const status = ref<UiShellStatus | null>(null);
  const loading = ref(true);
  const safeMode = ref(false);
  const switchingShellId = ref<UiShellId | null>(null);
  const restarting = ref(false);
  const showRestartDialog = ref(false);
  const errorMessage = ref<string | null>(null);
  const restartNotice = ref<string | null>(null);
  const browserOnly = computed(() => isBrowserEnv());

  const pendingShellName = computed(() => {
    const targetShellId = status.value?.configured_shell;
    if (!targetShellId) {
      return "";
    }

    const targetShell = shellCards.value.find((shell) => shell.id === targetShellId);
    return targetShell?.name ?? i18n.t(`plugins.ui_shell.shells.${targetShellId}.name`);
  });

  const shellCards = computed<UiShellCardViewModel[]>(() => {
    const shellMap = new Map<UiShellId, UiShellInfo>();

    for (const shellId of UI_SHELL_ORDER) {
      shellMap.set(shellId, createFallbackShell(shellId));
    }

    for (const shell of status.value?.available_shells ?? []) {
      shellMap.set(shell.id, shell);
    }

    return UI_SHELL_ORDER.map((shellId) => {
      const shell = shellMap.get(shellId) ?? createFallbackShell(shellId);
      const running = status.value?.effective_shell === shellId;
      const selected = status.value?.configured_shell === shellId;
      const pendingRestart = selected && status.value?.pending_restart === true;
      const disabledBySafeMode = safeMode.value && shellId === "next";
      const available = disabledBySafeMode ? false : shell.available;
      const disabled =
        disabledBySafeMode ||
        browserOnly.value ||
        !shell.available ||
        running ||
        switchingShellId.value !== null;

      const statusBadges: UiShellCardViewModel["statusBadges"] = [];
      if (running) {
        statusBadges.push({
          text: i18n.t("plugins.ui_shell.current_running"),
          variant: "success",
        });
      }
      if (pendingRestart) {
        statusBadges.push({
          text: i18n.t("plugins.ui_shell.pending_restart"),
          variant: "warning",
        });
      }
      if (disabledBySafeMode) {
        statusBadges.push({
          text: i18n.t("plugins.safe_mode_disabled"),
          variant: "neutral",
        });
      } else if (!shell.available) {
        statusBadges.push({
          text: i18n.t("plugins.ui_shell.unavailable"),
          variant: "neutral",
        });
      } else if (!running && !pendingRestart) {
        statusBadges.push({
          text: i18n.t("plugins.ui_shell.available_target"),
          variant: "info",
        });
      }

      let actionLabel = i18n.t("plugins.ui_shell.switch_action", {
        name: shell.name,
      });
      if (running) {
        actionLabel = i18n.t("plugins.ui_shell.current_running");
      } else if (pendingRestart) {
        actionLabel = i18n.t("plugins.ui_shell.restart_now");
      }

      return {
        id: shellId,
        name: shell.name,
        description: shell.description,
        sourceLabel: i18n.t(`plugins.ui_shell.source.${shell.source}`),
        running,
        selected,
        pendingRestart,
        available,
        disabled,
        actionLabel,
        statusBadges,
      };
    });
  });

  async function refreshStatus() {
    if (browserOnly.value) {
      loading.value = false;
      errorMessage.value = null;
      restartNotice.value = null;
      return;
    }

    loading.value = true;
    errorMessage.value = null;
    restartNotice.value = null;

    try {
      const [nextStatus, safeModeStatus] = await Promise.all([
        uiShellApi.getStatus(),
        systemApi.getSafeModeStatus(),
      ]);
      status.value = nextStatus;
      safeMode.value = safeModeStatus;
    } catch (error) {
      const normalized = normalizeAppError(error);
      errorMessage.value = normalized.message;
      pluginLogger.error("UiShellPanel", "UI 壳状态读取失败", normalized);
    } finally {
      loading.value = false;
    }
  }

  async function selectShell(shellId: UiShellId) {
    if (browserOnly.value) {
      return;
    }

    const selectedShell = shellCards.value.find((shell) => shell.id === shellId);
    if (!selectedShell || selectedShell.disabled) {
      return;
    }

    if (selectedShell.pendingRestart) {
      showRestartDialog.value = true;
      return;
    }

    switchingShellId.value = shellId;
    errorMessage.value = null;

    try {
      status.value = await uiShellApi.setShell(shellId);
      safeMode.value = await systemApi.getSafeModeStatus();
      if (status.value.pending_restart) {
        showRestartDialog.value = true;
      }
    } catch (error) {
      const normalized = normalizeAppError(error);
      errorMessage.value = normalized.message;
      pluginLogger.error("UiShellPanel", `UI 壳切换失败: ${shellId}`, normalized);
    } finally {
      switchingShellId.value = null;
    }
  }

  function requestRestart() {
    if (!status.value?.pending_restart || browserOnly.value) {
      return;
    }

    showRestartDialog.value = true;
  }

  async function confirmRestart() {
    restarting.value = true;
    errorMessage.value = null;
    restartNotice.value = null;

    try {
      await uiShellApi.restartApp();
    } catch (error) {
      const normalized = normalizeAppError(error);
      if (normalized.code === DEV_RESTART_MANUAL_REQUIRED_CODE) {
        restartNotice.value = i18n.t("plugins.ui_shell.dev_restart_manual_required_message");
        pluginLogger.info("UiShellPanel", "开发环境下跳过自动重启", normalized);
        return;
      }

      errorMessage.value = normalized.message;
      pluginLogger.error("UiShellPanel", "应用重启失败", normalized);
    } finally {
      restarting.value = false;
      showRestartDialog.value = false;
    }
  }

  onMounted(() => {
    refreshStatus();
  });

  return {
    shellCards,
    loading,
    safeMode,
    restarting,
    showRestartDialog,
    errorMessage,
    restartNotice,
    browserOnly,
    pendingShellName,
    hasPendingRestart: computed(() => status.value?.pending_restart === true),
    switchInFlight: computed(() => switchingShellId.value !== null),
    refreshStatus,
    selectShell,
    requestRestart,
    confirmRestart,
    closeRestartDialog: () => {
      showRestartDialog.value = false;
    },
  };
}
