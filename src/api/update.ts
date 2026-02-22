import { tauriInvoke } from "@api/tauri";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url?: string;
  release_notes?: string;
  published_at?: string;
  source?: string;
}

export interface PendingUpdate {
  version: string;
  date: string;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
}

export async function checkUpdate(): Promise<UpdateInfo | null> {
  try {
    const result = await tauriInvoke<UpdateInfo>("check_update");
    return result;
  } catch (error) {
    console.error("检查更新失败:", error);
    throw error;
  }
}

export async function downloadUpdate(url: string, expectedHash?: string): Promise<string> {
  return tauriInvoke<string>("download_update", { url, expectedHash });
}

export async function installUpdate(filePath: string, version: string): Promise<void> {
  return tauriInvoke<void>("install_update", { filePath, version });
}

export async function checkPendingUpdate(): Promise<PendingUpdate | null> {
  return tauriInvoke<PendingUpdate | null>("check_pending_update");
}

export async function clearPendingUpdate(): Promise<void> {
  return tauriInvoke<void>("clear_pending_update");
}

export async function restartAndInstall(): Promise<void> {
  return tauriInvoke<void>("restart_and_install");
}

export function onDownloadProgress(
  callback: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("update-download-progress", (event) => {
    callback(event.payload);
  });
}
