import { nextTick, ref, type Ref } from "vue";
import { m_pluginApi, type m_PluginConfigFile, type m_PluginInfo } from "@api/mcs_plugins";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";

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
  const selectedPlugin = ref<m_PluginInfo | null>(null);
  const loadedPlugins = ref<Set<string>>(new Set());
  const observer = ref<IntersectionObserver | null>(null);
  const pluginRowElements = ref<Map<string, HTMLElement>>(new Map());

  function removePluginFromState(pluginFileName: string) {
    plugins.value = plugins.value.filter((p) => p.file_name !== pluginFileName);
    if (selectedPlugin.value?.file_name === pluginFileName) {
      selectedPlugin.value = null;
    }
    loadedPlugins.value.delete(pluginFileName);
    pluginRowElements.value.delete(pluginFileName);
  }

  async function loadPlugins() {
    if (!options.currentServerId.value) return;

    pluginsLoading.value = true;
    options.setError(null);
    try {
      plugins.value = await m_pluginApi.m_getPlugins(options.currentServerId.value);
      loadedPlugins.value = new Set();
      nextTick(() => {
        setupIntersectionObserver();
      });
    } catch (e) {
      options.setError(String(e));
      plugins.value = [];
    } finally {
      pluginsLoading.value = false;
    }
  }

  function setupIntersectionObserver() {
    if (observer.value) {
      observer.value.disconnect();
    }

    observer.value = new IntersectionObserver(
      (intersectionEntries) => {
        intersectionEntries.forEach((entry) => {
          if (entry.isIntersecting) {
            const pluginElement = entry.target as HTMLElement;
            const pluginFileName = pluginElement.dataset.pluginFileName;
            if (pluginFileName) {
              void loadPluginDetails(pluginFileName);
            }
          }
        });
      },
      {
        rootMargin: "200px 0px",
        threshold: 0.1,
      },
    );

    pluginRowElements.value.forEach((element) => {
      observer.value?.observe(element);
    });
  }

  function registerPluginRow(payload: { pluginFileName: string; element: HTMLElement | null }) {
    const { pluginFileName, element } = payload;
    const previousElement = pluginRowElements.value.get(pluginFileName);

    if (previousElement) {
      observer.value?.unobserve(previousElement);
    }

    if (!element) {
      pluginRowElements.value.delete(pluginFileName);
      return;
    }

    element.dataset.pluginFileName = pluginFileName;
    pluginRowElements.value.set(pluginFileName, element);
    observer.value?.observe(element);
  }

  async function loadPluginDetails(pluginFileName: string) {
    if (loadedPlugins.value.has(pluginFileName)) {
      return;
    }

    loadedPlugins.value.add(pluginFileName);
  }

  async function togglePlugin(plugin: m_PluginInfo) {
    if (!options.currentServerId.value) return;

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
    try {
      await m_pluginApi.m_reloadPlugins(options.currentServerId.value);
      await loadPlugins();
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function handlePluginClick(plugin: m_PluginInfo) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) return;

    if (selectedPlugin.value?.file_name === plugin.file_name) {
      selectedPlugin.value = null;
      return;
    }

    if (!plugin.config_files || (plugin.config_files.length === 0 && plugin.has_config_folder)) {
      try {
        const configFiles = await m_pluginApi.m_getPluginConfigFiles(
          activeServerId,
          plugin.file_name,
          plugin.name,
        );
        const updatedPlugin = {
          ...plugin,
          config_files: configFiles,
        };
        selectedPlugin.value = updatedPlugin;
        const pluginIndex = plugins.value.findIndex((p) => p.file_name === plugin.file_name);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex] = updatedPlugin;
        }
      } catch (e) {
        console.error("Failed to load plugin config files:", e);
        selectedPlugin.value = plugin;
      }
      return;
    }

    selectedPlugin.value = plugin;
  }

  async function openPluginFolder(plugin: m_PluginInfo) {
    const server = options.getCurrentServer();
    if (!server) return;

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
    selectedPlugin,
    loadPlugins,
    reloadPlugins,
    handlePluginClick,
    togglePlugin,
    deletePlugin,
    registerPluginRow,
    openPluginFolder,
    openConfigFile,
    formatFileSize,
  };
}
