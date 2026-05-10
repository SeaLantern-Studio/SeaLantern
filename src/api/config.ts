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
   * 读取 server.properties 原始文本
   */
  async readServerPropertiesSource(serverPath: string): Promise<string> {
    return tauriInvoke("read_server_properties_source", {
      serverPath,
    });
  },

  /**
   * 直接写入 server.properties 原始文本
   */
  async writeServerPropertiesSource(serverPath: string, source: string): Promise<void> {
    return tauriInvoke("write_server_properties_source", {
      serverPath,
      source,
    });
  },

  /**
   * 将原始文本解析为可视化配置结构
   */
  async parseServerPropertiesSource(source: string): Promise<ServerProperties> {
    return tauriInvoke("parse_server_properties_source", {
      source,
    });
  },

  /**
   * 预览可视化配置写回后最终文本
   */
  async previewServerPropertiesWrite(
    serverPath: string,
    values: Record<string, string>,
  ): Promise<string> {
    return tauriInvoke("preview_server_properties_write", {
      serverPath,
      values,
    });
  },

  /**
   * 基于给定源码预览可视化配置写回后的最终文本
   */
  async previewServerPropertiesWriteFromSource(
    source: string,
    values: Record<string, string>,
  ): Promise<string> {
    return tauriInvoke("preview_server_properties_write_from_source", {
      source,
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
