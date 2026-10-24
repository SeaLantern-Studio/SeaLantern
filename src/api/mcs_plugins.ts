import { tauriInvoke } from "@api/tauri";

export interface m_PluginInfo {
  m_id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  file_name: string;
  file_size: number;
  enabled: boolean;
  main_class: string;
  has_config_folder: boolean;
  config_files: m_PluginConfigFile[];
}

export interface m_PluginConfigFile {
  file_name: string;
  content: string;
  file_type: string;
  file_path: string;
}

export const m_pluginApi = {
  async m_getPlugins(serverId: string): Promise<m_PluginInfo[]> {
    return tauriInvoke<m_PluginInfo[]>("m_get_plugins", { server_id: serverId });
  },

  async m_getPluginConfigFiles(
    serverId: string,
    fileName: string,
    pluginName: string,
  ): Promise<m_PluginConfigFile[]> {
    return tauriInvoke<m_PluginConfigFile[]>("m_get_plugin_config_files", {
      server_id: serverId,
      file_name: fileName,
      plugin_name: pluginName,
    });
  },

  async m_togglePlugin(serverId: string, fileName: string, enabled: boolean): Promise<void> {
    return tauriInvoke<void>("m_toggle_plugin", {
      server_id: serverId,
      file_name: fileName,
      enabled,
    });
  },

  async m_deletePlugin(serverId: string, fileName: string): Promise<void> {
    return tauriInvoke<void>("m_delete_plugin", { server_id: serverId, file_name: fileName });
  },

  async m_installPlugin(serverId: string, fileData: number[], fileName: string): Promise<void> {
    return tauriInvoke<void>("m_install_plugin", {
      server_id: serverId,
      file_name: fileName,
      file_data: fileData,
    });
  },

  async m_reloadPlugins(serverId: string): Promise<void> {
    // Send reload command to server
    return tauriInvoke<void>("send_command", { id: serverId, command: "reload" });
  },
};
