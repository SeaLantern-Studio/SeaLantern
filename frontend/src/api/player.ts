import { tauriInvoke } from "@api/tauri";

/**
 * 玩家条目
 */
export interface PlayerEntry {
  uuid: string;
  name: string;
}

/**
 * 封禁条目
 */
export interface BanEntry {
  uuid: string;
  name: string;
  reason: string;
  source: string;
  created: string;
  expires: string;
}

/**
 * OP (管理员) 条目
 */
export interface OpEntry {
  uuid: string;
  name: string;
  level: number;
  bypasses_player_limit: boolean;
}

/**
 * 玩家管理 API
 */
export const playerApi = {
  /**
   * 获取白名单 (从文件读取，任何时候都可以)
   */
  async getWhitelist(serverPath: string): Promise<PlayerEntry[]> {
    return tauriInvoke("get_whitelist", { serverPath });
  },

  /**
   * 获取封禁玩家列表
   */
  async getBannedPlayers(serverPath: string): Promise<BanEntry[]> {
    return tauriInvoke("get_banned_players", { serverPath });
  },

  /**
   * 获取 OP 列表
   */
  async getOps(serverPath: string): Promise<OpEntry[]> {
    return tauriInvoke("get_ops", { serverPath });
  },

  /**
   * 添加玩家到白名单 (向运行中的服务器发送命令)
   */
  async addToWhitelist(serverId: string, name: string): Promise<string> {
    return tauriInvoke("add_to_whitelist", { serverId, name });
  },

  /**
   * 从白名单移除玩家
   */
  async removeFromWhitelist(serverId: string, name: string): Promise<string> {
    return tauriInvoke("remove_from_whitelist", { serverId, name });
  },

  /**
   * 封禁玩家
   */
  async banPlayer(serverId: string, name: string, reason: string = ""): Promise<string> {
    return tauriInvoke("ban_player", { serverId, name, reason });
  },

  /**
   * 解封玩家
   */
  async unbanPlayer(serverId: string, name: string): Promise<string> {
    return tauriInvoke("unban_player", { serverId, name });
  },

  /**
   * 添加 OP
   */
  async addOp(serverId: string, name: string): Promise<string> {
    return tauriInvoke("add_op", { serverId, name });
  },

  /**
   * 移除 OP
   */
  async removeOp(serverId: string, name: string): Promise<string> {
    return tauriInvoke("remove_op", { serverId, name });
  },

  /**
   * 踢出玩家
   */
  async kickPlayer(serverId: string, name: string, reason: string = ""): Promise<string> {
    return tauriInvoke("kick_player", { serverId, name, reason });
  },

  /**
   * 导出日志
   */
  async exportLogs(logs: string[], savePath: string): Promise<void> {
    return tauriInvoke("export_logs", { logs, savePath });
  },
};
