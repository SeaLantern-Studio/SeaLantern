import { tauriInvoke } from "@api/tauri";

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
  file_path: string;
  version: string;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percent?: number;
}

export const updateMetadataApi = {
  async check(): Promise<UpdateInfo | null> {
    try {
      const result = await tauriInvoke<UpdateInfo>("check_update");
      return result;
    } catch (error) {
      console.error("检查更新失败:", error);
      throw error;
    }
  },
};
