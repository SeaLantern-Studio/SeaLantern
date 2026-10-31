import { tauriInvoke } from "@api/tauri";
import { invoke } from "@api/invoke";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url?: string;
  mirror_url?: string;
  sha256?: string;
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
    return await invoke<UpdateInfo>("check_update");
  } catch (error) {
    console.error("检查更新失败:", error);
    throw error;
  }
}

/**
 * 下载更新包并返回本地文件路径。
 *
 * `mirrorUrl` 放在参数末尾而非第二位，避免已有调用方按
 * `(url, expectedHash, version)` 传参时参数整体错位。
 */
export async function downloadUpdate(
  url: string,
  expectedHash?: string,
  version?: string,
  mirrorUrl?: string,
): Promise<string> {
  // 后端参数名是 snake_case，version 非可选需给默认值
  return invoke<string>("update_download", {
    url,
    mirror_url: mirrorUrl,
    expected_hash: expectedHash,
    version: version ?? "",
  });
}

export async function installUpdate(filePath: string, version: string): Promise<void> {
  // 后端 update_install 接收 file_path 和 arguments 数组
  return invoke<void>("update_install", {
    file_path: filePath,
    arguments: [version],
  });
}

export async function checkPendingUpdate(): Promise<PendingUpdate | null> {
  return invoke<PendingUpdate | null>("update_pending");
}

export async function clearPendingUpdate(): Promise<void> {
  return invoke<void>("update_clear_pending");
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
