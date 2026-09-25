import { ref, type Ref } from "vue";
import { open as openNativeDialog } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { serverPluginApi, type PluginConfigFile, type PluginSummary } from "@api/serverPlugin";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import { isUploadSupported, pickFileFromBrowser } from "@api/upload";
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

/** 取路径的父目录，兼容 Windows 与 POSIX 两种分隔符 */
function getParentDir(filePath: string): string {
  const trimmed = filePath.replace(/[/\\]+$/, "");
  const lastSeparator = Math.max(trimmed.lastIndexOf("/"), trimmed.lastIndexOf("\\"));
  return lastSeparator > 0 ? trimmed.slice(0, lastSeparator) : "";
}

/** 拼接路径前校验单个路径分量：插件名来自 jar 内的 plugin.yml，可能被构造成路径穿越 */
function isSafePathSegment(segment: string): boolean {
  return (
    segment.length > 0 &&
    segment !== "." &&
    segment !== ".." &&
    !segment.includes("/") &&
    !segment.includes("\\")
  );
}

export function useConfigPlugins(options: UseConfigPluginsOptions) {
  const plugins = ref<PluginSummary[]>([]);
  const pluginsLoading = ref(false);
  const selectedPlugin = ref<PluginSummary | null>(null);
  /** 行元素仅用于删除时的收起动画：列表数据一次性返回，不依赖进入视口懒加载 */
  const pluginRowElements = ref<Map<string, HTMLElement>>(new Map());

  function removePluginFromState(pluginFileName: string) {
    plugins.value = plugins.value.filter((p) => p.file_name !== pluginFileName);
    if (selectedPlugin.value?.file_name === pluginFileName) {
      selectedPlugin.value = null;
    }
    pluginRowElements.value.delete(pluginFileName);
  }

  async function loadPlugins() {
    if (!options.currentServerId.value) return;

    pluginsLoading.value = true;
    options.setError(null);
    try {
      plugins.value = await serverPluginApi.listServerPlugins(options.currentServerId.value);
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

    pluginRowElements.value.set(pluginFileName, element);
  }

  async function togglePlugin(plugin: PluginSummary) {
    if (!options.currentServerId.value) return;

    if (!plugin.file_name.endsWith(".jar") && !plugin.file_name.endsWith(".jar.disabled")) {
      alert(i18n.t("config.not_jar_file", { file: plugin.file_name }));
      return;
    }

    try {
      await serverPluginApi.setServerPluginEnabled(
        options.currentServerId.value,
        plugin.file_name,
        !plugin.enabled,
      );
      plugin.enabled = !plugin.enabled;
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function deletePlugin(plugin: PluginSummary) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) return;

    try {
      const pluginElement = pluginRowElements.value.get(plugin.file_name);
      if (pluginElement) {
        const originalHeight = pluginElement.offsetHeight;
        pluginElement.style.height = `${originalHeight}px`;
        pluginElement.style.flexShrink = "0";
        pluginElement.classList.add("deleting");

        setTimeout(async () => {
          try {
            await serverPluginApi.deleteServerPlugin(activeServerId, plugin.file_name);
            removePluginFromState(plugin.file_name);
          } catch (e) {
            options.setError(String(e));
          }
        }, 500);
      } else {
        await serverPluginApi.deleteServerPlugin(activeServerId, plugin.file_name);
        removePluginFromState(plugin.file_name);
      }
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function reloadPlugins() {
    if (!options.currentServerId.value) return;
    try {
      // 重载由服务端自身处理：这里只是把 reload 命令送进控制台。
      await serverApi.sendCommand(options.currentServerId.value, "reload");
      await loadPlugins();
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function handlePluginClick(plugin: PluginSummary) {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) return;

    if (selectedPlugin.value?.file_name === plugin.file_name) {
      selectedPlugin.value = null;
      return;
    }

    if (!plugin.config_files || (plugin.config_files.length === 0 && plugin.has_config_folder)) {
      try {
        const configFiles = await serverPluginApi.readServerPluginConfigFiles(
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

  async function openPluginFolder(plugin: PluginSummary) {
    if (!isSafePathSegment(plugin.name)) {
      options.setError(i18n.t("config.invalid_plugin_name", { name: plugin.name }));
      return;
    }

    const server = options.getCurrentServer();
    if (!server) return;

    const basePath = server.path.replace(/[/\\]+$/, "");
    const separator = basePath.includes("\\") ? "\\" : "/";
    const pluginConfigPath = `${basePath}${separator}plugins${separator}${plugin.name}`;

    try {
      await systemApi.openFolder(pluginConfigPath);
    } catch (e) {
      options.setError(String(e));
    }
  }

  async function openConfigFile(config: PluginConfigFile) {
    // 后端未注册 open_file 命令，这里退化为打开配置文件所在的目录
    const parentDir = getParentDir(config.file_path);
    if (!parentDir) {
      options.setError(i18n.t("config.open_config_file_failed"));
      return;
    }

    try {
      await systemApi.openFolder(parentDir);
    } catch (e) {
      options.setError(String(e));
    }
  }

  /** 读取待安装 jar 的字节内容：浏览器取 File 对象，桌面走原生对话框加文件系统 */
  async function pickJarBytes(): Promise<{ fileName: string; fileData: number[] } | null> {
    if (isUploadSupported()) {
      const picked = await pickFileFromBrowser({ accept: ".jar" });
      const file = Array.isArray(picked) ? picked[0] : picked;
      if (!file) return null;

      const buffer = await file.arrayBuffer();
      return { fileName: file.name, fileData: Array.from(new Uint8Array(buffer)) };
    }

    const selected = await openNativeDialog({
      multiple: false,
      filters: [{ name: "Plugin", extensions: ["jar"] }],
    });
    if (!selected || Array.isArray(selected)) return null;

    const fileName = selected.split(/[/\\]/).pop() || "";
    if (!fileName) return null;

    const buffer = await readFile(selected);
    return { fileName, fileData: Array.from(buffer) };
  }

  /** 选择本地 jar 安装到当前实例；后端只接受纯文件名，同名插件会被直接覆盖，这里先确认 */
  async function installPluginFromLocal() {
    const activeServerId = options.currentServerId.value;
    if (!activeServerId) return;

    try {
      const picked = await pickJarBytes();
      if (!picked) return;

      if (!picked.fileName.toLowerCase().endsWith(".jar")) {
        options.setError(i18n.t("config.not_jar_file", { file: picked.fileName }));
        return;
      }

      const overwriting = plugins.value.some((p) => p.file_name === picked.fileName);
      if (
        overwriting &&
        !window.confirm(i18n.t("config.install_plugin_overwrite", { file: picked.fileName }))
      ) {
        return;
      }

      await serverPluginApi.installServerPlugin(activeServerId, picked.fileData, picked.fileName);
      await loadPlugins();
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
    installPluginFromLocal,
    formatFileSize,
  };
}
