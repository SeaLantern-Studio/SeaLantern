import { tauriInvoke } from "@api/tauri";

/** 备份内容选项 */
export type BackupContentType = "core" | "config" | "plugins" | "world" | "logs";

/** 备份项信息 */
export interface BackupItem {
  id: string;
  serverId: string;
  name: string;
  size: number; // 字节
  createdAt: string; // ISO 时间
  contents: BackupContentType[];
}

/** 备份设置 */
export interface BackupSettings {
  maxBackups: number; // 最多保留备份数
  autoBackupEnabled: boolean;
  autoBackupInterval: number; // 小时数
  autoBackupContents: BackupContentType[];
}

/** 创建备份请求 */
export interface CreateBackupRequest {
  serverId: string;
  contents: BackupContentType[];
  name?: string;
}

/** 备份 API */
export const backupApi = {
  /** 获取服务器备份列表 */
  async list(serverId: string): Promise<BackupItem[]> {
    return tauriInvoke("get_backup_list", { serverId });
  },

  /** 创建备份 */
  async create(request: CreateBackupRequest): Promise<BackupItem> {
    return tauriInvoke("create_backup", request);
  },

  /** 删除备份 */
  async delete(backupId: string): Promise<void> {
    return tauriInvoke("delete_backup", { backupId });
  },

  /** 恢复备份 */
  async restore(backupId: string): Promise<void> {
    return tauriInvoke("restore_backup", { backupId });
  },

  /** 获取备份设置 */
  async getSettings(serverId: string): Promise<BackupSettings> {
    return tauriInvoke("get_backup_settings", { serverId });
  },

  /** 更新备份设置 */
  async updateSettings(serverId: string, settings: Partial<BackupSettings>): Promise<void> {
    return tauriInvoke("update_backup_settings", { serverId, settings });
  },
};
