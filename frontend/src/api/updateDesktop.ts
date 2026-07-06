import { assertDesktopEnvironment, isBrowserEnv, tauriInvokeDesktop } from "@api/tauri";
import type { DownloadProgress, PendingUpdate } from "@api/update";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const desktopUpdaterApi = {
  async download(url: string, expectedHash?: string, version?: string): Promise<string> {
    return tauriInvokeDesktop<string>("download_update", { url, expectedHash, version });
  },

  async install(filePath: string, version: string): Promise<void> {
    return tauriInvokeDesktop<void>("install_update", { filePath, version });
  },

  async checkPending(): Promise<PendingUpdate | null> {
    return tauriInvokeDesktop<PendingUpdate | null>("check_pending_update");
  },

  async clearPending(): Promise<void> {
    return tauriInvokeDesktop<void>("clear_pending_update");
  },

  async restartAndInstall(): Promise<void> {
    return tauriInvokeDesktop<void>("restart_and_install");
  },

  async downloadFromDebugUrl(url: string): Promise<string> {
    return tauriInvokeDesktop<string>("download_update_from_debug_url", { url });
  },

  onDownloadProgress(callback: (progress: DownloadProgress) => void): Promise<UnlistenFn> {
    assertDesktopEnvironment();

    if (isBrowserEnv()) {
      return Promise.resolve(() => {});
    }

    return listen<DownloadProgress>("update-download-progress", (event) => {
      callback(event.payload);
    });
  },
};
