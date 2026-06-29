import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import { createPluginLifecycleLoaders } from "./pluginLifecycleLoaders";
import { createPluginLifecycleMutations } from "./pluginLifecycleMutations";
import {
  type CreatePluginLifecycleActionsOptions,
  type SetStoreError,
} from "./pluginLifecycleShared";
import { createPluginLifecycleToggleAction } from "./pluginLifecycleToggle";

export function createPluginLifecycleActions(options: CreatePluginLifecycleActionsOptions) {
  const setStoreError: SetStoreError = (message, errorCause) => {
    const normalized = normalizeAppError(errorCause);
    options.error.value = normalized.message;
    pluginLogger.error("Store", message, normalized);
    return normalized.message;
  };

  const loaders = createPluginLifecycleLoaders(options, { setStoreError });
  const toggle = createPluginLifecycleToggleAction(options, {
    loadNavItems: loaders.loadNavItems,
  });
  const mutations = createPluginLifecycleMutations(options, {
    loadPlugins: loaders.loadPlugins,
    setStoreError,
  });

  return {
    loadPlugins: loaders.loadPlugins,
    refreshPlugins: loaders.refreshPlugins,
    loadNavItems: loaders.loadNavItems,
    togglePlugin: toggle.togglePlugin,
    confirmEnablePlugin: toggle.confirmEnablePlugin,
    installFromZip: mutations.installFromZip,
    installBatch: mutations.installBatch,
    deletePlugin: mutations.deletePlugin,
    deletePlugins: mutations.deletePlugins,
    checkUpdate: mutations.checkUpdate,
    checkAllUpdates: mutations.checkAllUpdates,
  };
}
