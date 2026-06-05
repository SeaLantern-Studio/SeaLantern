import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import {
  cleanupPluginRuntime,
  currentPluginPath,
  setPluginState,
  type CreatePluginLifecycleActionsOptions,
  type TogglePluginResult,
} from "./pluginLifecycleShared";

interface PluginLifecycleToggleDependencies {
  loadNavItems: () => Promise<void>;
}

function collectThemeProviderIdsForEnable(
  options: Pick<CreatePluginLifecycleActionsOptions, "plugins">,
  pluginId: string,
): string[] {
  return options.plugins.value
    .filter((plugin) => plugin.manifest.id === pluginId || plugin.state === "enabled")
    .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
    .map((plugin) => plugin.manifest.id);
}

function collectThemeProviderIdsForDisable(
  options: Pick<CreatePluginLifecycleActionsOptions, "plugins">,
  pluginId: string,
  disabledPlugins: string[],
): string[] {
  return options.plugins.value
    .filter((plugin) => plugin.manifest.id !== pluginId)
    .filter((plugin) => !disabledPlugins.includes(plugin.manifest.id))
    .filter((plugin) => plugin.state === "enabled")
    .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
    .map((plugin) => plugin.manifest.id);
}

function disablePluginRuntime(
  options: Pick<
    CreatePluginLifecycleActionsOptions,
    | "plugins"
    | "removePluginCss"
    | "removePluginUiElements"
    | "cleanupPluginEventListeners"
    | "removePluginProxies"
    | "removePluginComponents"
  >,
  pluginIds: string[],
) {
  for (const pluginId of pluginIds) {
    cleanupPluginRuntime(options, pluginId);
    setPluginState(options, pluginId, "disabled");
  }
}

export function createPluginLifecycleToggleAction(
  options: CreatePluginLifecycleActionsOptions,
  dependencies: PluginLifecycleToggleDependencies,
) {
  async function togglePlugin(pluginId: string, enable: boolean): Promise<TogglePluginResult> {
    if (enable) {
      try {
        await pluginApi.enablePlugin(pluginId);

        options.syncThemeProviderOverrides(collectThemeProviderIdsForEnable(options, pluginId));
        await options.injectPluginCss(pluginId);
        setPluginState(options, pluginId, "enabled");

        options.syncThemeProviderOverrides();
        await dependencies.loadNavItems();
        await pluginApi.onPageChanged(currentPluginPath());
        await options.replayUiSnapshot();
        setTimeout(() => void options.replayUiSnapshot(), 300);

        return { success: true };
      } catch (errorCause) {
        const normalized = normalizeAppError(errorCause);
        setPluginState(options, pluginId, "disabled");
        options.removePluginCss(pluginId);
        options.syncThemeProviderOverrides();
        pluginLogger.error("Store", `插件启用失败: ${pluginId}`, normalized);
        return { success: false, error: normalized.message };
      }
    }

    try {
      const disabledPlugins = await pluginApi.disablePlugin(pluginId);
      options.syncThemeProviderOverrides(
        collectThemeProviderIdsForDisable(options, pluginId, disabledPlugins),
      );

      const allDisabledPluginIds = [pluginId, ...disabledPlugins.filter((id) => id !== pluginId)];
      disablePluginRuntime(options, allDisabledPluginIds);

      options.syncThemeProviderOverrides();
      await dependencies.loadNavItems();
      return { success: true, disabledPlugins };
    } catch (errorCause) {
      const normalized = normalizeAppError(errorCause);
      pluginLogger.error("Store", `插件停用失败: ${pluginId}`, normalized);
      return { success: false, error: normalized.message };
    }
  }

  return {
    togglePlugin,
  };
}
