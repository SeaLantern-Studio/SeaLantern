import { tauriInvoke } from "@api/tauri";

/**
 * 配置条目
 */
export interface ConfigEntry {
  key: string;
  value: string;
  description: string;
  value_type: string;
  default_value: string;
  category: string;
}

/**
 * 服务器配置文件
 */
export interface ServerProperties {
  entries: ConfigEntry[];
  raw: Record<string, string>;
}

/**
 * 配置管理 API
 */
export const configApi = {
  /**
   * 读取服务器配置文件 (server.properties)
   */
  async readServerProperties(serverPath: string): Promise<ServerProperties> {
    return tauriInvoke("read_server_properties", {
      serverPath,
    });
  },

  /**
   * 写入服务器配置文件
   */
  async writeServerProperties(serverPath: string, values: Record<string, string>): Promise<void> {
    return tauriInvoke("write_server_properties", {
      serverPath,
      values,
    });
  },

  /**
   * 读取通用配置文件
   */
  async readConfig(serverPath: string, path: string): Promise<Record<string, string>> {
    return tauriInvoke("read_config", { serverPath, path });
  },

  /**
   * 写入通用配置文件
   */
  async writeConfig(
    serverPath: string,
    path: string,
    values: Record<string, string>,
  ): Promise<void> {
    return tauriInvoke("write_config", { serverPath, path, values });
  },
};
