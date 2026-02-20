import { tauriInvoke } from "./tauri";

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  file_name: string;
  file_size: number;
  enabled: boolean;
  main_class: string;
  has_config_folder: boolean;
  config_files: PluginConfigFile[];
}

export interface PluginConfigFile {
  file_name: string;
  content: string;
  file_type: string;
  file_path: string;
}

export const pluginApi = {
  async getPlugins(serverId: string): Promise<PluginInfo[]> {
    return tauriInvoke<PluginInfo[]>("get_plugins", { serverId });
  },

  async togglePlugin(serverId: string, fileName: string, enabled: boolean): Promise<void> {
    return tauriInvoke<void>("toggle_plugin", { serverId, fileName, enabled });
  },

  async deletePlugin(serverId: string, fileName: string): Promise<void> {
    return tauriInvoke<void>("delete_plugin", { serverId, fileName });
  },

  async installPlugin(serverId: string, fileData: number[], fileName: string): Promise<void> {
    return tauriInvoke<void>("install_plugin", { serverId, fileData, fileName });
  },

  async reloadPlugins(serverId: string): Promise<void> {
    // Send reload command to server
    return tauriInvoke<void>("send_command", { id: serverId, command: "reload" });
  },
};