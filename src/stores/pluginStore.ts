import { defineStore } from "pinia";
import { ref } from "vue";
import { isBrowserEnv } from "@api/tauri";
import { createPluginAppearanceManager } from "@stores/plugin/pluginAppearanceManager";
import { createPluginComponentBridge } from "@stores/plugin/pluginComponentBridge";
import { createPluginLifecycleActions } from "@stores/plugin/pluginLifecycleActions";
import { createPluginRuntimeUiBridge } from "@stores/plugin/pluginRuntimeUiBridge";
import { createPluginSnapshotReplay } from "@stores/plugin/pluginSnapshotReplay";
import {
  createPluginCapabilityManager,
  createPluginSettingsWriter,
  readPluginSettings,
} from "@stores/plugin/pluginSettingsManager";
import { createPluginTelemetryBridge } from "@stores/plugin/pluginTelemetryBridge";
import type {
  PluginInfo,
  PluginNavItem,
  PluginUpdateInfo,
  MissingDependency,
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

  const capabilityManager = createPluginCapabilityManager({
    getPlugins: () => plugins.value,
  });
  const settingsWriter = createPluginSettingsWriter({
    getPlugins: () => plugins.value,
    applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings,
  });
  const getPluginSettings = readPluginSettings;
  const { setPluginSettings } = settingsWriter;
  const { hasCapability } = capabilityManager;

  function collectSidebarItems() {
    // 已禁用插件注册的侧栏按钮功能
    sidebarItems.value = [];
  }

  async function loadPluginIcons() {
    await loadPluginIconsFromAppearance();
  }

  const {
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
  } = createPluginLifecycleActions({
    plugins,
    navItems,
    loading,
    error,
    icons,
    updates,
    pendingDependencies,
    syncThemeProviderOverrides,
    loadPluginIcons,
    injectAllPluginCss,
    injectPluginCss,
    removePluginCss,
    removePluginUiElements,
    cleanupPluginEventListeners,
    removePluginProxies,
    removePluginComponents,
    replayUiSnapshot,
    collectSidebarItems,
  });

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
