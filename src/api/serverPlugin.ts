import { tauriInvoke } from "@api/tauri";

/** 一个已安装服务器插件的摘要信息。 */
export interface PluginSummary {
  /** 插件标识，形如 `名称-版本`。 */
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  /** jar 文件名，始终不含 `.disabled` 后缀。 */
  file_name: string;
  file_size: number;
  /** 是否处于启用状态。 */
  enabled: boolean;
  main_class: string;
  has_config_folder: boolean;
  /** 列表接口恒为空数组，点击插件后再按需读取。 */
  config_files: PluginConfigFile[];
}

/** 插件配置目录中的一个可读文本文件。 */
export interface PluginConfigFile {
  file_name: string;
  content: string;
  file_type: string;
  /** 文件绝对路径，供系统程序打开。 */
  file_path: string;
}

export const serverPluginApi = {
  /** 列出实例的全部服务器插件（含已禁用）。 */
  async listServerPlugins(instanceId: string): Promise<PluginSummary[]> {
    return tauriInvoke<PluginSummary[]>("list_server_plugins", { instanceId });
  },

  /** 读取某个插件配置目录下的文本文件。 */
  async readServerPluginConfigFiles(
    instanceId: string,
    fileName: string,
    pluginName: string,
  ): Promise<PluginConfigFile[]> {
    return tauriInvoke<PluginConfigFile[]>("read_server_plugin_config_files", {
      instanceId,
      fileName,
      pluginName,
    });
  },

  /** 启用或禁用一个插件（重命名 `.jar` ↔ `.jar.disabled`）。 */
  async setServerPluginEnabled(
    instanceId: string,
    fileName: string,
    enabled: boolean,
  ): Promise<void> {
    return tauriInvoke<void>("set_server_plugin_enabled", { instanceId, fileName, enabled });
  },

  /** 永久删除一个插件。 */
  async deleteServerPlugin(instanceId: string, fileName: string): Promise<void> {
    return tauriInvoke<void>("delete_server_plugin", { instanceId, fileName });
  },

  /** 安装一个插件：字节内容由前端一次性给出。 */
  async installServerPlugin(
    instanceId: string,
    fileData: number[],
    fileName: string,
  ): Promise<void> {
    return tauriInvoke<void>("install_server_plugin", { instanceId, fileName, fileData });
  },
};
