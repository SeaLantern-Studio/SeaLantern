import { type UiShellId, settingsApi } from "@api/settings";
import { systemApi } from "@api/system";
import { tauriInvoke } from "@api/tauri";
import { uiShellApi } from "@api/uiShell";

const HEARTBEAT_INTERVAL = 5000;

let heartbeatStarted = false;

export interface DesktopShellSelection {
  configuredShell: UiShellId;
  effectiveShell: UiShellId;
  safeMode: boolean;
}

export async function resolveDesktopShellSelection(): Promise<DesktopShellSelection> {
  const [settings, safeMode] = await Promise.all([
    settingsApi.get(),
    systemApi.getSafeModeStatus(),
  ]);
  const configuredShell = settings.ui_shell;

  return {
    configuredShell,
    effectiveShell: safeMode ? "classic" : configuredShell,
    safeMode,
  };
}

export function startDesktopHeartbeat(): void {
  if (heartbeatStarted) {
    return;
  }

  heartbeatStarted = true;

  setInterval(() => {
    tauriInvoke("frontend_heartbeat", undefined, { silent: true }).catch(() => {
      // Backend may have exited already.
    });
  }, HEARTBEAT_INTERVAL);
}

export async function reportDesktopShellRuntime(shellId: UiShellId): Promise<void> {
  try {
    await uiShellApi.reportRuntime(shellId);
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn("Failed to report desktop UI shell runtime", error);
    }
  }
}
