import { tauriInvoke } from './tauri';

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url?: string;
  release_notes?: string;
  published_at?: string;
  source?: string;
}

export async function checkUpdate(): Promise<UpdateInfo | null> {
  try {
    const result = await tauriInvoke<UpdateInfo>('check_update');
    return result;
  } catch (error) {
    console.error('检查更新失败:', error);
    throw error;
  }
}
