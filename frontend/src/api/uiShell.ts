import { tauriInvoke } from "@api/tauri";
import type { UiShellId } from "@api/settings";

export interface UiShellInfo {
  id: UiShellId;
  name: string;
  description: string;
  source: "builtin";
  builtin: boolean;
  available: boolean;
}

export interface UiShellStatus {
  configured_shell: UiShellId;
  effective_shell: UiShellId;
  pending_restart: boolean;
  available_shells: UiShellInfo[];
}

export const uiShellApi = {
  async getStatus(): Promise<UiShellStatus> {
    return tauriInvoke("get_ui_shell_status");
  },

  async setShell(shellId: UiShellId): Promise<UiShellStatus> {
    return tauriInvoke("set_ui_shell", {
      request: { shell_id: shellId },
    });
  },

  async reportRuntime(shellId: UiShellId): Promise<UiShellStatus> {
    return tauriInvoke("report_ui_shell_runtime", {
      request: { shell_id: shellId },
    });
  },

  async restartApp(): Promise<void> {
    return tauriInvoke("restart_app");
  },
};
