import { tauriInvoke } from "@api/tauri";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";

export type ServerConfigFileKind = "properties" | "toml" | "yaml" | "json";

export type ServerConfigOwnership = "service_managed" | "server_managed" | "third_party";

export type ServerConfigSourceKind = "server_root" | "manual_root" | "manual_file";

export type ServerConfigSearchMode = "keyword" | "regex" | "similarity";

export type ServerConfigSearchScope = "path" | "content" | "all";

export type ServerConfigJsonMode = "disabled" | "filtered" | "all";

export type KnownServerConfigRole =
  | "startup_primary"
  | "startup_legacy"
  | "server_properties"
  | "pumpkin";

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

export interface DiscoveredServerConfigFile {
  locator: string;
  relative_path: string;
  file_name: string;
  absolute_path: string;
  source_kind: ServerConfigSourceKind;
  source_label: string;
  server_relative_path?: string | null;
  kind: ServerConfigFileKind;
  known_role?: KnownServerConfigRole | null;
  ownership: ServerConfigOwnership;
  priority: number;
}

export interface ServerConfigSearchHit {
  locator: string;
  relative_path: string;
  file_name: string;
  absolute_path: string;
  source_kind: ServerConfigSourceKind;
  source_label: string;
  server_relative_path?: string | null;
  kind: ServerConfigFileKind;
  known_role?: KnownServerConfigRole | null;
  ownership: ServerConfigOwnership;
  priority: number;
  score: number;
  reason: string;
  content_match?: ServerConfigContentMatch | null;
}

export interface ServerConfigContentMatch {
  line_number: number;
  line_text: string;
}

export interface ServerConfigDiscoveryOptions {
  manual_import_dirs: string[];
  manual_import_files: string[];
  json_mode: ServerConfigJsonMode;
}

export interface ServerConfigDocument {
  relative_path: string;
  kind: ServerConfigFileKind;
  content: unknown;
}

/**
 * SL.json 启动配置
 */
export interface SLStartupConfig {
  max_memory: number | null;
  min_memory: number | null;
  jvm_args: string[];
  cpu_policy: CpuPolicyConfig;
  jvm_preset: JvmPresetConfig;
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

  async listServerConfigFiles(
    serverPath: string,
    discoveryOptions?: ServerConfigDiscoveryOptions,
  ): Promise<DiscoveredServerConfigFile[]> {
    return tauriInvoke("list_server_config_files", { serverPath, discoveryOptions });
  },

  async searchServerConfigFiles(
    serverPath: string,
    query: string,
    mode: ServerConfigSearchMode,
    scope: ServerConfigSearchScope,
    discoveryOptions?: ServerConfigDiscoveryOptions,
    limit?: number,
    caseSensitive = false,
  ): Promise<ServerConfigSearchHit[]> {
    return tauriInvoke("search_server_config_files", {
      serverPath,
      query,
      mode,
      scope,
      discoveryOptions,
      limit,
      caseSensitive,
    });
  },

  async readServerConfigSource(
    serverPath: string,
    relativePath: string,
    locator?: string,
    discoveryOptions?: ServerConfigDiscoveryOptions,
  ): Promise<string> {
    return tauriInvoke("read_server_config_source", {
      serverPath,
      relativePath,
      locator,
      discoveryOptions,
    });
  },

  async writeServerConfigSource(
    serverPath: string,
    relativePath: string,
    source: string,
    locator?: string,
    discoveryOptions?: ServerConfigDiscoveryOptions,
  ): Promise<void> {
    return tauriInvoke("write_server_config_source", {
      serverPath,
      relativePath,
      locator,
      discoveryOptions,
      source,
    });
  },

  async readServerConfigDocument(
    serverPath: string,
    relativePath: string,
    locator?: string,
    discoveryOptions?: ServerConfigDiscoveryOptions,
  ): Promise<ServerConfigDocument> {
    return tauriInvoke("read_server_config_document", {
      serverPath,
      relativePath,
      locator,
      discoveryOptions,
    });
  },

  async writeServerConfigDocument(
    serverPath: string,
    relativePath: string,
    content: unknown,
    locator?: string,
    discoveryOptions?: ServerConfigDiscoveryOptions,
  ): Promise<void> {
    return tauriInvoke("write_server_config_document", {
      serverPath,
      relativePath,
      locator,
      discoveryOptions,
      content,
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

  /**
   * 读取 SL.json 启动配置
   */
  async readSLConfig(serverPath: string): Promise<SLStartupConfig> {
    return tauriInvoke("read_sl_config", { serverPath });
  },

  /**
   * 写入 SL.json 启动配置
   */
  async writeSLConfig(serverPath: string, config: SLStartupConfig): Promise<void> {
    return tauriInvoke("write_sl_config", { serverPath, config });
  },
};
