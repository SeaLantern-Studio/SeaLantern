import { tauriInvoke } from "@api/tauri";
import { rpcInvoke } from "@api/rpc";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url?: string;
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
    return await rpcInvoke<UpdateInfo>("update.check");
  } catch (error) {
    console.error("检查更新失败:", error);
    throw error;
  }
}

export async function downloadUpdate(
  url: string,
  expectedHash?: string,
  version?: string,
): Promise<string> {
  // 后端参数名是 snake_case，version 非可选需给默认值
  return rpcInvoke<string>("update.download", {
    url,
    expected_hash: expectedHash,
    version: version ?? "",
  });
}

export async function installUpdate(filePath: string, version: string): Promise<void> {
  // 后端 update_install 接收 file_path 和 arguments 数组
  return rpcInvoke<void>("update.install", {
    file_path: filePath,
    arguments: [version],
  });
}

export async function checkPendingUpdate(): Promise<PendingUpdate | null> {
  return rpcInvoke<PendingUpdate | null>("update.pending");
}

export async function clearPendingUpdate(): Promise<void> {
  return rpcInvoke<void>("update.clearPending");
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
