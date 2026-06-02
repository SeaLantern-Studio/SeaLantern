import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type {
  BatchInstallResult,
  MissingDependency,
  PluginInfo,
  PluginNavItem,
  PluginUpdateInfo,
  PluginInstallResult,
} from "@type/plugin";
import type { Ref } from "vue";

interface CreatePluginLifecycleActionsOptions {
  plugins: Ref<PluginInfo[]>;
  navItems: Ref<PluginNavItem[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  icons: Ref<Record<string, string>>;
  updates: Ref<Record<string, PluginUpdateInfo>>;
  pendingDependencies: Ref<MissingDependency[]>;
  syncThemeProviderOverrides: (pluginIds?: string[]) => void;
  loadPluginIcons: () => Promise<void>;
  injectAllPluginCss: () => Promise<void>;
  injectPluginCss: (pluginId: string) => Promise<void>;
  removePluginCss: (pluginId: string) => void;
  removePluginUiElements: (pluginId: string) => void;
  cleanupPluginEventListeners: (pluginId: string) => void;
  removePluginProxies: (pluginId: string) => void;
  removePluginComponents: (pluginId: string) => void;
  replayUiSnapshot: () => Promise<void>;
  collectSidebarItems: () => void;
}

function currentPluginPath() {
  return window.location.hash.replace(/^#/, "") || "/";
}

export function createPluginLifecycleActions(options: CreatePluginLifecycleActionsOptions) {
  function setStoreError(message: string, errorCause: unknown): string {
    const normalized = normalizeAppError(errorCause);
    options.error.value = normalized.message;
    pluginLogger.error("Store", message, normalized);
    return normalized.message;
  }

  async function loadPlugins() {
    options.loading.value = true;
    options.error.value = null;
    try {
      options.plugins.value = await pluginApi.listPlugins();
      options.syncThemeProviderOverrides();
      await options.loadPluginIcons();
      await options.injectAllPluginCss();
      options.collectSidebarItems();
      await options.replayUiSnapshot();
    } catch (errorCause) {
      setStoreError("插件列表读取失败", errorCause);
    } finally {
      options.loading.value = false;
    }
  }

  async function refreshPlugins() {
    options.loading.value = true;
    options.error.value = null;
    try {
      options.plugins.value = await pluginApi.scanPlugins();
      options.syncThemeProviderOverrides();
      await options.loadPluginIcons();
      await options.injectAllPluginCss();
      options.collectSidebarItems();
      await options.replayUiSnapshot();
    } catch (errorCause) {
      setStoreError("插件列表刷新失败", errorCause);
    } finally {
      options.loading.value = false;
    }
  }

  async function loadNavItems() {
    try {
      options.navItems.value = await pluginApi.getPluginNavItems();
      options.collectSidebarItems();
    } catch (errorCause) {
      pluginLogger.error("Store", "插件导航读取失败", normalizeAppError(errorCause));
    }
  }

  async function togglePlugin(
    pluginId: string,
    enable: boolean,
  ): Promise<{ success: boolean; error?: string; disabledPlugins?: string[] }> {
    if (enable) {
      try {
        await pluginApi.enablePlugin(pluginId);

        options.syncThemeProviderOverrides(
          options.plugins.value
            .filter((plugin) => plugin.manifest.id === pluginId || plugin.state === "enabled")
            .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
            .map((plugin) => plugin.manifest.id),
        );

        await options.injectPluginCss(pluginId);

        const pluginIndex = options.plugins.value.findIndex(
          (plugin) => plugin.manifest.id === pluginId,
        );
        if (pluginIndex !== -1) {
          options.plugins.value[pluginIndex].state = "enabled";
        }

        options.syncThemeProviderOverrides();
        await loadNavItems();
        await pluginApi.onPageChanged(currentPluginPath());
        await options.replayUiSnapshot();
        setTimeout(() => void options.replayUiSnapshot(), 300);

        return { success: true };
      } catch (errorCause) {
        const normalized = normalizeAppError(errorCause);
        const pluginIndex = options.plugins.value.findIndex(
          (plugin) => plugin.manifest.id === pluginId,
        );
        if (pluginIndex !== -1) {
          options.plugins.value[pluginIndex].state = "disabled";
          options.removePluginCss(pluginId);
        }

        options.syncThemeProviderOverrides();
        pluginLogger.error("Store", `插件启用失败: ${pluginId}`, normalized);
        return { success: false, error: normalized.message };
      }
    }

    try {
      const disabledPlugins = await pluginApi.disablePlugin(pluginId);

      options.syncThemeProviderOverrides(
        options.plugins.value
          .filter((plugin) => plugin.manifest.id !== pluginId)
          .filter((plugin) => !disabledPlugins.includes(plugin.manifest.id))
          .filter((plugin) => plugin.state === "enabled")
          .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
          .map((plugin) => plugin.manifest.id),
      );

      options.removePluginCss(pluginId);
      options.removePluginUiElements(pluginId);
      options.cleanupPluginEventListeners(pluginId);
      options.removePluginProxies(pluginId);
      options.removePluginComponents(pluginId);

      const pluginIndex = options.plugins.value.findIndex(
        (plugin) => plugin.manifest.id === pluginId,
      );
      if (pluginIndex !== -1) {
        options.plugins.value[pluginIndex].state = "disabled";
      }

      options.syncThemeProviderOverrides();

      for (const disabledId of disabledPlugins) {
        const index = options.plugins.value.findIndex(
          (plugin) => plugin.manifest.id === disabledId,
        );
        if (index !== -1) {
          options.plugins.value[index].state = "disabled";
          options.removePluginCss(disabledId);
          options.removePluginUiElements(disabledId);
          options.cleanupPluginEventListeners(disabledId);
          options.removePluginProxies(disabledId);
          options.removePluginComponents(disabledId);
        }
      }

      await loadNavItems();
      return { success: true, disabledPlugins };
    } catch (errorCause) {
      const normalized = normalizeAppError(errorCause);
      pluginLogger.error("Store", `插件停用失败: ${pluginId}`, normalized);
      return { success: false, error: normalized.message };
    }
  }

  async function installFromZip(zipPath: string): Promise<PluginInstallResult> {
    options.loading.value = true;
    try {
      const result = await pluginApi.installPlugin(zipPath);
      if (result.missing_dependencies.length > 0) {
        options.pendingDependencies.value = result.missing_dependencies;
      }
      await loadPlugins();
      return result;
    } catch (errorCause) {
      pluginLogger.error("Store", "插件安装失败", normalizeAppError(errorCause));
      throw errorCause;
    } finally {
      options.loading.value = false;
    }
  }

  async function installBatch(paths: string[]): Promise<BatchInstallResult> {
    options.loading.value = true;
    try {
      const result = await pluginApi.installPluginsBatch(paths);
      if (result.failed.length > 0) {
        for (const item of result.failed) {
          pluginLogger.error("Store", `批量安装失败: ${item.path}`, item.error);
        }
      }
      await loadPlugins();
      return result;
    } catch (errorCause) {
      pluginLogger.error("Store", "批量安装执行失败", normalizeAppError(errorCause));
      throw errorCause;
    } finally {
      options.loading.value = false;
    }
  }

  async function deletePlugin(pluginId: string, deleteData?: boolean) {
    try {
      await pluginApi.deletePlugin(pluginId, deleteData);
      delete options.icons.value[pluginId];
      delete options.updates.value[pluginId];
      options.removePluginCss(pluginId);
      await loadPlugins();
    } catch (errorCause) {
      setStoreError(`插件删除失败: ${pluginId}`, errorCause);
      throw errorCause;
    }
  }

  async function deletePlugins(pluginIds: string[], deleteData?: boolean) {
    try {
      await pluginApi.deletePlugins(pluginIds, deleteData);
      for (const pluginId of pluginIds) {
        delete options.icons.value[pluginId];
        delete options.updates.value[pluginId];
        options.removePluginCss(pluginId);
      }
      await loadPlugins();
    } catch (errorCause) {
      setStoreError("插件批量删除失败", errorCause);
      throw errorCause;
    }
  }

  async function checkUpdate(pluginId: string) {
    try {
      const update = await pluginApi.checkPluginUpdate(pluginId);
      if (update) {
        options.updates.value[pluginId] = update;
      }
      return update;
    } catch (errorCause) {
      pluginLogger.error("Store", `插件更新检查失败: ${pluginId}`, normalizeAppError(errorCause));
      return null;
    }
  }

  async function checkAllUpdates() {
    try {
      const allUpdates = await pluginApi.checkAllPluginUpdates();
      for (const update of allUpdates) {
        options.updates.value[update.plugin_id] = update;
      }
      return allUpdates;
    } catch (errorCause) {
      pluginLogger.error("Store", "插件批量更新检查失败", normalizeAppError(errorCause));
      return [];
    }
  }

  return {
    loadPlugins,
    refreshPlugins,
    loadNavItems,
    togglePlugin,
    installFromZip,
    installBatch,
    deletePlugin,
    deletePlugins,
    checkUpdate,
    checkAllUpdates,
  };
}
