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
 * 玩家档案（按用户名查询 UUID 的结果，来源为服务器本地 usercache.json）
 */
export interface PlayerProfile {
  name: string;
  uuid: string;
}

/**
 * 玩家管理 API
 *
 * 参数键名约定：所有 `invoke` 调用统一用 snake_case，与后端
 * `#[tauri::command(rename_all = "snake_case")]` 对齐。其它子模块
 * (instance / console / cron / download 等) 同样遵守此约定。
 */
export const playerApi = {
  /**
   * 获取在线玩家列表（向运行中的服务器发送 list 命令并解析回显）
   */
  async getOnlinePlayers(serverId: string): Promise<string[]> {
    return tauriInvoke("get_online_players", { server_id: serverId });
  },

  /**
   * 获取白名单（向服务器发送 whitelist list 命令，UUID 由 usercache 反查）
   */
  async getWhitelist(serverId: string, serverPath: string): Promise<PlayerEntry[]> {
    return tauriInvoke("get_whitelist", { server_id: serverId, server_path: serverPath });
  },

  /**
   * 获取封禁玩家列表
   */
  async getBannedPlayers(serverId: string, serverPath: string): Promise<BanEntry[]> {
    return tauriInvoke("get_banned_players", { server_id: serverId, server_path: serverPath });
  },

  /**
   * 获取 OP 列表（当前为 list 输出里 * 前缀的在线玩家）
   */
  async getOps(serverId: string, serverPath: string): Promise<OpEntry[]> {
    return tauriInvoke("get_ops", { server_id: serverId, server_path: serverPath });
  },

  /**
   * 添加玩家到白名单 (向运行中的服务器发送命令)
   */
  async addToWhitelist(serverId: string, name: string): Promise<string> {
    return tauriInvoke("add_to_whitelist", { server_id: serverId, name });
  },

  /**
   * 从白名单移除玩家
   */
  async removeFromWhitelist(serverId: string, name: string): Promise<string> {
    return tauriInvoke("remove_from_whitelist", { server_id: serverId, name });
  },

  /**
   * 封禁玩家
   */
  async banPlayer(serverId: string, name: string, reason: string = ""): Promise<string> {
    return tauriInvoke("ban_player", { server_id: serverId, name, reason });
  },

  /**
   * 解封玩家
   */
  async unbanPlayer(serverId: string, name: string): Promise<string> {
    return tauriInvoke("unban_player", { server_id: serverId, name });
  },

  /**
   * 添加 OP
   */
  async addOp(serverId: string, name: string): Promise<string> {
    return tauriInvoke("add_op", { server_id: serverId, name });
  },

  /**
   * 移除 OP
   */
  async removeOp(serverId: string, name: string): Promise<string> {
    return tauriInvoke("remove_op", { server_id: serverId, name });
  },

  /**
   * 踢出玩家
   */
  async kickPlayer(serverId: string, name: string, reason: string = ""): Promise<string> {
    return tauriInvoke("kick_player", { server_id: serverId, name, reason });
  },

  /**
   * 导出日志
   */
  async exportLogs(logs: string[], savePath: string): Promise<void> {
    return tauriInvoke("export_logs", { logs, save_path: savePath });
  },

  /**
   * 按用户名查询玩家档案（UUID），从服务器本地 usercache.json 读取
   */
  async lookupPlayer(serverPath: string, username: string): Promise<PlayerProfile> {
    return tauriInvoke("lookup_player", { server_path: serverPath, username });
  },
};
