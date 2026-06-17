import { ref, type Ref } from "vue";
import { m_pluginApi, type m_PluginConfigFile, type m_PluginInfo } from "@api/mcs_plugins";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";
import {
  getPluginUnsupportedReason,
  serverSupportsPluginExtensions,
} from "@utils/serverExtensionSupport";
import { useConfigPluginSelection } from "@views/config/useConfigPluginSelection";

interface UseConfigPluginsOptions {
  currentServerId: Ref<string | null>;
  getCurrentServer: () => ServerInstance | null;
  setError: (message: string | null) => void;
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function useConfigPlugins(options: UseConfigPluginsOptions) {
  const plugins = ref<m_PluginInfo[]>([]);
  const pluginsLoading = ref(false);
  const pluginsSupported = ref(true);
  const pluginsUnsupportedReason = ref<string | null>(null);
  const pluginRowElements = ref<Map<string, HTMLElement>>(new Map());
  const selection = useConfigPluginSelection({
    currentServerId: options.currentServerId,
    plugins,
    setError: options.setError,
  });

  function removePluginFromState(pluginFileName: string) {
    plugins.value = plugins.value.filter((p) => p.file_name !== pluginFileName);
    selection.clearSelectedPlugin(pluginFileName);
    pluginRowElements.value.delete(pluginFileName);
  }

  function syncPluginSupportState() {
    const server = options.getCurrentServer();
    if (!server) {
      pluginsSupported.value = true;
      pluginsUnsupportedReason.value = null;
      return true;
    }

    const supported = serverSupportsPluginExtensions(server);
    pluginsSupported.value = supported;
    pluginsUnsupportedReason.value = supported ? null : getPluginUnsupportedReason(server);
    if (!supported) {
      plugins.value = [];
      selection.clearSelection();
    }
    return supported;
  }

  async function loadPlugins() {
    if (!options.currentServerId.value) return;
    if (!syncPluginSupportState()) {
      options.setError(null);
      return;
    }

    pluginsLoading.value = true;
    options.setError(null);
    try {
      plugins.value = await m_pluginApi.m_getPlugins(options.currentServerId.value);
    } catch (e) {
      options.setError(String(e));
      plugins.value = [];
    } finally {
      pluginsLoading.value = false;
    }
  }

  function registerPluginRow(payload: { pluginFileName: string; element: HTMLElement | null }) {
    const { pluginFileName, element } = payload;

    if (!element) {
      pluginRowElements.value.delete(pluginFileName);
      return;
    }

    element.dataset.pluginFileName = pluginFileName;
    pluginRowElements.value.set(pluginFileName, element);
  }

  async function togglePlugin(plugin: m_PluginInfo) {
    if (!options.currentServerId.value) return;
    if (!syncPluginSupportState()) return;

    if (!plugin.file_name.endsWith(".jar") && !plugin.file_name.endsWith(".jar.disabled")) {
      alert(i18n.t("config.not_jar_file", { file: plugin.file_name }));
      return;
    }

    try {
      await m_pluginApi.m_togglePlugin(
        options.currentServerId.value,
        plugin.file_name,
        !plugin.enabled,
      );
      plugin.enabled = !plugin.enabled;
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function deletePlugin(plugin: m_PluginInfo) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) return;
    if (!syncPluginSupportState()) return;

    try {
      const pluginElement = pluginRowElements.value.get(plugin.file_name);
      if (pluginElement) {
        const originalHeight = pluginElement.offsetHeight;
        pluginElement.style.height = `${originalHeight}px`;
        pluginElement.style.flexShrink = "0";
        pluginElement.dataset.pluginFileName = plugin.file_name;

        pluginElement.classList.add("deleting");

        setTimeout(async () => {
          await m_pluginApi.m_deletePlugin(activeServerId, plugin.file_name);
          removePluginFromState(plugin.file_name);
        }, 500);
      } else {
        await m_pluginApi.m_deletePlugin(activeServerId, plugin.file_name);
        removePluginFromState(plugin.file_name);
      }
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function reloadPlugins() {
    if (!options.currentServerId.value) return;
    if (!syncPluginSupportState()) return;
    try {
      await m_pluginApi.m_reloadPlugins(options.currentServerId.value);
      await loadPlugins();
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function openPluginFolder(plugin: m_PluginInfo) {
    const server = options.getCurrentServer();
    if (!server) return;
    if (!syncPluginSupportState()) return;

    const basePath = server.path.replace(/[/\\]$/, "");
    const separator = basePath.includes("\\") ? "\\" : "/";
    const pluginConfigPath = `${basePath}${separator}plugins${separator}${plugin.name}`;

    try {
      await systemApi.openFolder(pluginConfigPath);
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function openConfigFile(config: m_PluginConfigFile) {
    try {
      await systemApi.openFile(config.file_path);
    } catch (e) {
      options.setError(String(e));
    }
  }

  return {
    plugins,
    pluginsLoading,
    pluginsSupported,
    pluginsUnsupportedReason,
    selectedPlugin: selection.selectedPlugin,
    loadPlugins,
    reloadPlugins,
    handlePluginClick: selection.handlePluginClick,
    togglePlugin,
    deletePlugin,
    registerPluginRow,
    openPluginFolder,
    openConfigFile,
    formatFileSize,
  };
}
