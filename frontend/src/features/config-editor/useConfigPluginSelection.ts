import { ref, type Ref } from "vue";
import { m_pluginApi, type m_PluginInfo } from "@api/mcs_plugins";

interface UseConfigPluginSelectionOptions {
  currentServerId: Ref<string | null>;
  plugins: Ref<m_PluginInfo[]>;
  setError: (message: string | null) => void;
}

function shouldLoadConfigFiles(plugin: m_PluginInfo) {
  return !plugin.config_files || (plugin.config_files.length === 0 && plugin.has_config_folder);
}

export function useConfigPluginSelection(options: UseConfigPluginSelectionOptions) {
  const selectedPlugin = ref<m_PluginInfo | null>(null);

  function clearSelection() {
    selectedPlugin.value = null;
  }

  function clearSelectedPlugin(pluginFileName: string) {
    if (selectedPlugin.value?.file_name === pluginFileName) {
      selectedPlugin.value = null;
    }
  }

  function replacePluginInList(updatedPlugin: m_PluginInfo) {
    const pluginIndex = options.plugins.value.findIndex(
      (plugin) => plugin.file_name === updatedPlugin.file_name,
    );
    if (pluginIndex !== -1) {
      options.plugins.value[pluginIndex] = updatedPlugin;
    }
  }

  async function loadPluginDetails(plugin: m_PluginInfo) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) {
      return plugin;
    }

    const configFiles = await m_pluginApi.m_getPluginConfigFiles(
      activeServerId,
      plugin.file_name,
      plugin.name,
    );

    const updatedPlugin = {
      ...plugin,
      config_files: configFiles,
    };
    replacePluginInList(updatedPlugin);
    return updatedPlugin;
  }

  async function handlePluginClick(plugin: m_PluginInfo) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) {
      return;
    }

    if (selectedPlugin.value?.file_name === plugin.file_name) {
      selectedPlugin.value = null;
      return;
    }

    if (!shouldLoadConfigFiles(plugin)) {
      selectedPlugin.value = plugin;
      return;
    }

    try {
      selectedPlugin.value = await loadPluginDetails(plugin);
    } catch (e) {
      options.setError(String(e));
      selectedPlugin.value = plugin;
    }
  }

  return {
    selectedPlugin,
    clearSelection,
    clearSelectedPlugin,
    handlePluginClick,
  };
}
