import { tauriInvoke } from "./tauri";

export interface ConfigEntry {
  key: string;
  value: string;
  description: string;
  value_type: string;
  default_value: string;
  category: string;
}

export interface ServerProperties {
  entries: ConfigEntry[];
  raw: Record<string, string>;
}

export const configApi = {
  async readServerProperties(serverPath: string): Promise<ServerProperties> {
    return tauriInvoke("read_server_properties", {
      serverPath,
    });
  },

  async writeServerProperties(serverPath: string, values: Record<string, string>): Promise<void> {
    return tauriInvoke("write_server_properties", {
      serverPath,
      values,
    });
  },

  async readConfig(path: string): Promise<Record<string, string>> {
    return tauriInvoke("read_config", { path });
  },

  async writeConfig(path: string, values: Record<string, string>): Promise<void> {
    return tauriInvoke("write_config", { path, values });
  },
};
