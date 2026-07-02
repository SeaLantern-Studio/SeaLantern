import type { UiShellId } from "@api/settings";
import { tauriInvoke } from "@api/tauri";
import { uiShellApi } from "@api/uiShell";

const HEARTBEAT_INTERVAL = 5000;
export const DESKTOP_PRIMARY_SHELL: UiShellId = "next";

let heartbeatStarted = false;

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
