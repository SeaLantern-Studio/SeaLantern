import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type { CreatePluginLifecycleActionsOptions, SetStoreError } from "./pluginLifecycleShared";

interface PluginLifecycleLoadersDependencies {
  setStoreError: SetStoreError;
}

export function createPluginLifecycleLoaders(
  options: CreatePluginLifecycleActionsOptions,
  dependencies: PluginLifecycleLoadersDependencies,
) {
  async function hydratePlugins(
    fetchPlugins: () => Promise<typeof options.plugins.value>,
    errorMessage: string,
  ) {
    options.loading.value = true;
    options.error.value = null;

    try {
      options.plugins.value = await fetchPlugins();
      options.syncThemeProviderOverrides();
      await options.loadPluginIcons();
      await options.injectAllPluginCss();
      options.collectSidebarItems();
      await options.replayUiSnapshot();
    } catch (errorCause) {
      dependencies.setStoreError(errorMessage, errorCause);
    } finally {
      options.loading.value = false;
    }
  }

  async function loadPlugins() {
    await hydratePlugins(() => pluginApi.listPlugins(), "插件列表读取失败");
  }

  async function refreshPlugins() {
    await hydratePlugins(() => pluginApi.scanPlugins(), "插件列表刷新失败");
  }

  async function loadNavItems() {
    try {
      options.navItems.value = await pluginApi.getPluginNavItems();
      options.collectSidebarItems();
    } catch (errorCause) {
      pluginLogger.error("Store", "插件导航读取失败", normalizeAppError(errorCause));
    }
  }

  return {
    loadPlugins,
    refreshPlugins,
    loadNavItems,
  };
}
