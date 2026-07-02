import { tauriInvoke } from "@api/tauri";
import type { UiShellId } from "@api/settings";

export const uiShellApi = {
  async reportRuntime(shellId: UiShellId): Promise<void> {
    return tauriInvoke("report_ui_shell_runtime", {
      request: { shell_id: shellId },
    });
  },
};
