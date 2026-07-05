export type ActiveUiShellId = "next";
import { tauriInvoke } from "@api/tauri";

const HEARTBEAT_INTERVAL = 5000;
export const DESKTOP_PRIMARY_SHELL: ActiveUiShellId = "next";

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
