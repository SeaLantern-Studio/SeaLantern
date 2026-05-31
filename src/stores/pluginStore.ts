import { defineStore } from "pinia";
import { ref } from "vue";
import { isBrowserEnv } from "@api/tauri";
import * as pluginApi from "@api/plugin";
import { createPluginAppearanceManager } from "@stores/plugin/pluginAppearanceManager";
import { createPluginComponentBridge } from "@stores/plugin/pluginComponentBridge";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { createPluginRuntimeUiBridge } from "@stores/plugin/pluginRuntimeUiBridge";
import { createPluginSnapshotReplay } from "@stores/plugin/pluginSnapshotReplay";
import { createPluginTelemetryBridge } from "@stores/plugin/pluginTelemetryBridge";
import { normalizeAppError } from "@utils/appError";
import type {
  PluginInfo,
  PluginNavItem,
  PluginUpdateInfo,
  PluginInstallResult,
  MissingDependency,
  BatchInstallResult,
  SidebarItem,
} from "@type/plugin";

export const usePluginStore = defineStore("plugin", () => {
  const plugins = ref<PluginInfo[]>([]);
  const navItems = ref<PluginNavItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const icons = ref<Record<string, string>>({});
  const updates = ref<Record<string, PluginUpdateInfo>>({});

  const pendingDependencies = ref<MissingDependency[]>([]);

  const sidebarItems = ref<SidebarItem[]>([]);

  const telemetryBridge = createPluginTelemetryBridge(() => plugins.value, isBrowserEnv);
  const {
    permissionLogs,
    pluginLogs,
    getPluginLogs,
    getPermissionLogs,
    clearPermissionLogs,
    getHighRiskPermissions,
    initPermissionLogListener,
    cleanupPermissionLogListener,
    initPluginLogListener,
    cleanupPluginLogListener,
    initI18nEventListener,
    cleanupI18nEventListener,
    hydratePermissionLogs,
    slicePermissionLogs,
  } = telemetryBridge;

  const runtimeUiBridge = createPluginRuntimeUiBridge({ isBrowserEnv });
  const {
    sanitizeCss,
    handlePluginUiEvent,
    removePluginUiElements,
    cleanupPluginEventListeners,
    initUiEventListener,
    cleanupUiEventListener,
    initSidebarEventListener,
    cleanupSidebarEventListener,
  } = runtimeUiBridge;

  const componentBridge = createPluginComponentBridge(() => plugins.value, isBrowserEnv);
  const {
    handlePluginComponentEvent,
    initComponentEventListener,
    cleanupComponentEventListener,
    removePluginProxies,
    removePluginComponents,
    consumePendingComponentCreates,
    consumePendingComponentDeletes,
  } = componentBridge;

  const snapshotReplay = createPluginSnapshotReplay({
    getPlugins: () => plugins.value,
    getSidebarItems: () => sidebarItems.value,
    setSidebarItems: (items) => {
      sidebarItems.value = items;
    },
    handlePluginUiEvent,
    handlePluginComponentEvent,
    hydratePermissionLogs,
    slicePermissionLogs,
  });
  const { replayUiSnapshot } = snapshotReplay;

  function setStoreError(message: string, errorCause: unknown): string {
    const normalized = normalizeAppError(errorCause);
    error.value = normalized.message;
    pluginLogger.error("Store", message, normalized);
    return normalized.message;
  }

  async function readPluginSettings(pluginId: string): Promise<Record<string, unknown>> {
    try {
      return await pluginApi.getPluginSettings(pluginId);
    } catch (errorCause) {
      pluginLogger.error("Store", `插件设置读取失败: ${pluginId}`, normalizeAppError(errorCause));
      return {};
    }
  }

  const appearanceManager = createPluginAppearanceManager({
    getPlugins: () => plugins.value,
    getIcons: () => icons.value,
    setIcon: (pluginId, iconData) => {
      icons.value[pluginId] = iconData;
    },
    sanitizeCss,
    getPluginSettings: readPluginSettings,
  });
  const {
    syncThemeProviderOverrides,
    loadPluginIcons: loadPluginIconsFromAppearance,
    injectPluginCss,
    removePluginCss,
    injectAllPluginCss,
    applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings,
  } = appearanceManager;

  async function loadPlugins() {
    loading.value = true;
    error.value = null;
    try {
      plugins.value = await pluginApi.listPlugins();
      syncThemeProviderOverrides();
      await loadPluginIcons();
      await injectAllPluginCss();

      collectSidebarItems();

      await replayUiSnapshot();
    } catch (errorCause) {
      setStoreError("插件列表读取失败", errorCause);
    } finally {
      loading.value = false;
    }
  }

  async function refreshPlugins() {
    loading.value = true;
    error.value = null;
    try {
      plugins.value = await pluginApi.scanPlugins();
      syncThemeProviderOverrides();
      await loadPluginIcons();
      await injectAllPluginCss();

      collectSidebarItems();

      await replayUiSnapshot();
    } catch (errorCause) {
      setStoreError("插件列表刷新失败", errorCause);
    } finally {
      loading.value = false;
    }
  }

  async function togglePlugin(
    pluginId: string,
    enable: boolean,
  ): Promise<{ success: boolean; error?: string; disabledPlugins?: string[] }> {
    if (enable) {
      try {
        await pluginApi.enablePlugin(pluginId);

        syncThemeProviderOverrides(
          plugins.value
            .filter((p) => p.manifest.id === pluginId || p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        await injectPluginCss(pluginId);

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "enabled";
        }

        syncThemeProviderOverrides();
        await loadNavItems();

        const currentPath = window.location.hash.replace(/^#/, "") || "/";
        await pluginApi.onPageChanged(currentPath);
        await replayUiSnapshot();
        setTimeout(() => replayUiSnapshot(), 300);

        return { success: true };
      } catch (errorCause) {
        const errorMessage = normalizeAppError(errorCause).message;

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "disabled";
          removePluginCss(pluginId);
        }

        syncThemeProviderOverrides();
        pluginLogger.error("Store", `插件启用失败: ${pluginId}`, normalizeAppError(errorCause));

        return { success: false, error: errorMessage };
      }
    } else {
      try {
        const disabledPlugins = await pluginApi.disablePlugin(pluginId);

        syncThemeProviderOverrides(
          plugins.value
            .filter((p) => p.manifest.id !== pluginId)
            .filter((p) => !disabledPlugins.includes(p.manifest.id))
            .filter((p) => p.state === "enabled")
            .filter((p) => p.manifest.capabilities?.includes("theme-provider"))
            .map((p) => p.manifest.id),
        );

        removePluginCss(pluginId);
        removePluginUiElements(pluginId);
        cleanupPluginEventListeners(pluginId);
        removePluginProxies(pluginId);
        removePluginComponents(pluginId);

        const pluginIndex = plugins.value.findIndex((p) => p.manifest.id === pluginId);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex].state = "disabled";
        }

        syncThemeProviderOverrides();

        for (const disabledId of disabledPlugins) {
          const idx = plugins.value.findIndex((p) => p.manifest.id === disabledId);
          if (idx !== -1) {
            plugins.value[idx].state = "disabled";
            removePluginCss(disabledId);
            removePluginUiElements(disabledId);
            cleanupPluginEventListeners(disabledId);
            removePluginProxies(disabledId);
            removePluginComponents(disabledId);
          }
        }

        await loadNavItems();
        return { success: true, disabledPlugins };
      } catch (errorCause) {
        const errorMessage = normalizeAppError(errorCause).message;
        pluginLogger.error("Store", `插件停用失败: ${pluginId}`, normalizeAppError(errorCause));
        return { success: false, error: errorMessage };
      }
    }
  }

  async function loadNavItems() {
    try {
      navItems.value = await pluginApi.getPluginNavItems();

      collectSidebarItems();
    } catch (e) {
      pluginLogger.error("Store", "插件导航读取失败", e);
    }
  }

  function collectSidebarItems() {
    // 已禁用插件注册的侧栏按钮功能
    sidebarItems.value = [];
  }

  async function installFromZip(zipPath: string): Promise<PluginInstallResult> {
    loading.value = true;
    try {
      const result = await pluginApi.installPlugin(zipPath);

      if (result.missing_dependencies.length > 0) {
        pendingDependencies.value = result.missing_dependencies;
      }
      await loadPlugins();
      return result;
    } catch (e) {
      pluginLogger.error("Store", "插件安装失败", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function installBatch(paths: string[]): Promise<BatchInstallResult> {
    loading.value = true;
    try {
      const result = await pluginApi.installPluginsBatch(paths);
      if (result.failed.length > 0) {
        for (const item of result.failed) {
          pluginLogger.error("Store", `批量安装失败: ${item.path}`, item.error);
        }
      }
      await loadPlugins();
      return result;
    } catch (e) {
      pluginLogger.error("Store", "批量安装执行失败", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadPluginIcons() {
    await loadPluginIconsFromAppearance();
  }

  async function getPluginSettings(pluginId: string): Promise<Record<string, unknown>> {
    return readPluginSettings(pluginId);
  }

  async function setPluginSettings(
    pluginId: string,
    settings: Record<string, unknown>,
  ): Promise<void> {
    try {
      await pluginApi.setPluginSettings(pluginId, settings);

      if (hasCapability(pluginId, "theme-provider")) {
        await applyThemeProviderSettings(pluginId);
      }
      if (hasCapability(pluginId, "theme-widgets-provider")) {
        await applyThemeWidgetsProviderSettings(pluginId);
      }
    } catch (e) {
      pluginLogger.error("Store", `插件设置保存失败: ${pluginId}`, e);
      throw e;
    }
  }

  function hasCapability(pluginId: string, capability: string): boolean {
    const plugin = plugins.value.find((p) => p.manifest.id === pluginId);
    return plugin?.manifest.capabilities?.includes(capability) ?? false;
  }

  async function deletePlugin(pluginId: string, deleteData?: boolean) {
    try {
      await pluginApi.deletePlugin(pluginId, deleteData);
      delete icons.value[pluginId];
      delete updates.value[pluginId];
      removePluginCss(pluginId);
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
        delete icons.value[pluginId];
        delete updates.value[pluginId];
        removePluginCss(pluginId);
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
        updates.value[pluginId] = update;
      }
      return update;
    } catch (e) {
      pluginLogger.error("Store", `插件更新检查失败: ${pluginId}`, e);
      return null;
    }
  }

  async function checkAllUpdates() {
    try {
      const allUpdates = await pluginApi.checkAllPluginUpdates();
      for (const update of allUpdates) {
        updates.value[update.plugin_id] = update;
      }
      return allUpdates;
    } catch (e) {
      pluginLogger.error("Store", "插件批量更新检查失败", e);
      return [];
    }
  }

  return {
    plugins,
    navItems,
    loading,
    error,
    icons,
    updates,
    pendingDependencies,
    sidebarItems,
    permissionLogs,
    pluginLogs,
    loadPlugins,
    refreshPlugins,
    togglePlugin,
    loadNavItems,
    collectSidebarItems,
    installFromZip,
    installBatch,
    loadPluginIcons,
    getPluginSettings,
    setPluginSettings,
    injectPluginCss,
    removePluginCss,
    injectAllPluginCss,
    deletePlugin,
    deletePlugins,
    checkUpdate,
    checkAllUpdates,
    applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings,
    hasCapability,

    initUiEventListener,
    cleanupUiEventListener,
    removePluginUiElements,

    initSidebarEventListener,
    cleanupSidebarEventListener,

    getPermissionLogs,
    clearPermissionLogs,
    initPermissionLogListener,
    cleanupPermissionLogListener,

    initPluginLogListener,
    cleanupPluginLogListener,
    getPluginLogs,

    getHighRiskPermissions,

    initComponentEventListener,
    cleanupComponentEventListener,
    removePluginProxies,
    removePluginComponents,
    consumePendingComponentCreates,
    consumePendingComponentDeletes,

    initI18nEventListener,
    cleanupI18nEventListener,

    replayUiSnapshot,
  };
});
