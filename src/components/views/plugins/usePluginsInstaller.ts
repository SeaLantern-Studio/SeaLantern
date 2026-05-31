import { onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { systemApi } from "@api/system";
import { isUploadSupported } from "@api/upload";
import { pickAndInstallPluginFiles, pickAndInstallPluginFolderLikeFile } from "@api/plugin";
import { usePluginStore } from "@stores/pluginStore";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type { BatchInstallResult, MissingDependency } from "@type/plugin";

export function usePluginsInstaller() {
  const pluginStore = usePluginStore();
  const isDragging = ref(false);
  const chooserOpen = ref(false);
  const safeMode = ref(false);
  const isInstalling = ref(false);
  const showDependencyModal = ref(false);
  const missingDependencies = ref<MissingDependency[]>([]);
  const installedPluginName = ref("");
  const showBatchResultModal = ref(false);
  const batchInstallResult = ref<BatchInstallResult | null>(null);
  const installErrorMessage = ref<string | null>(null);
  let unlistenDragDrop: (() => void) | null = null;

  async function handleInstall(filePath: string) {
    isInstalling.value = true;
    installErrorMessage.value = null;
    try {
      const result = await pluginStore.installFromZip(filePath);
      if (result.missing_dependencies.length > 0) {
        installedPluginName.value = result.plugin.manifest.name;
        missingDependencies.value = result.missing_dependencies;
        showDependencyModal.value = true;
      }
    } catch (error) {
      const normalized = normalizeAppError(error);
      installErrorMessage.value = normalized.message;
      pluginLogger.error("Installer", `插件安装失败: ${filePath}`, normalized);
    } finally {
      isInstalling.value = false;
    }
  }

  async function handleBatchInstall(paths: string[]) {
    if (paths.length === 1) {
      await handleInstall(paths[0]);
      return;
    }

    isInstalling.value = true;
    installErrorMessage.value = null;
    try {
      batchInstallResult.value = await pluginStore.installBatch(paths);
      showBatchResultModal.value = true;
      pluginLogger.info("Installer", "插件批量安装完成", {
        total: paths.length,
        success: batchInstallResult.value.success.length,
        failed: batchInstallResult.value.failed.length,
      });
    } catch (error) {
      const normalized = normalizeAppError(error);
      installErrorMessage.value = normalized.message;
      pluginLogger.error("Installer", "插件批量安装失败", {
        paths,
        error: normalized,
      });
    } finally {
      isInstalling.value = false;
    }
  }

  function openChooser() {
    chooserOpen.value = true;
  }

  async function pickFile() {
    chooserOpen.value = false;

    if (isUploadSupported()) {
      try {
        installErrorMessage.value = null;
        const result = await pickAndInstallPluginFiles();
        if (result) {
          batchInstallResult.value = result;
          showBatchResultModal.value = true;
        }
      } catch (error) {
        const normalized = normalizeAppError(error);
        installErrorMessage.value = normalized.message;
        pluginLogger.error("Installer", "浏览器环境插件文件安装失败", normalized);
      }
      return;
    }

    const selected = await open({
      multiple: true,
      filters: [{ name: "Plugin", extensions: ["zip", "json"] }],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        await handleBatchInstall(paths);
      }
    }
  }

  async function pickFolder() {
    chooserOpen.value = false;

    if (isUploadSupported()) {
      try {
        installErrorMessage.value = null;
        const result = await pickAndInstallPluginFolderLikeFile();
        if (result?.missing_dependencies?.length) {
          installedPluginName.value = result.plugin.manifest.name;
          missingDependencies.value = result.missing_dependencies;
          showDependencyModal.value = true;
        }
      } catch (error) {
        const normalized = normalizeAppError(error);
        installErrorMessage.value = normalized.message;
        pluginLogger.error("Installer", "浏览器环境插件目录安装失败", normalized);
      }
      return;
    }

    const selected = await open({ directory: true, multiple: true });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        await handleBatchInstall(paths);
      }
    }
  }

  onMounted(async () => {
    try {
      safeMode.value = await systemApi.getSafeModeStatus();
    } catch (error) {
      pluginLogger.error("Installer", "安全模式状态读取失败", normalizeAppError(error));
    }

    if (pluginStore.plugins.length === 0 && !pluginStore.loading) {
      pluginStore.loadPlugins();
    }

    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === "over") {
        isDragging.value = true;
        return;
      }

      if (event.payload.type === "drop") {
        isDragging.value = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          const validPaths = paths.filter(
            (path) =>
              path.endsWith(".zip") || path.endsWith("manifest.json") || !path.includes("."),
          );
          if (validPaths.length > 0) {
            await handleBatchInstall(validPaths);
          }
        }
        return;
      }

      isDragging.value = false;
    });
  });

  onUnmounted(() => {
    unlistenDragDrop?.();
  });

  return {
    isDragging,
    chooserOpen,
    safeMode,
    isInstalling,
    showDependencyModal,
    missingDependencies,
    installedPluginName,
    showBatchResultModal,
    batchInstallResult,
    installErrorMessage,
    openChooser,
    handleInstall,
    handleBatchInstall,
    pickFile,
    pickFolder,
  };
}
