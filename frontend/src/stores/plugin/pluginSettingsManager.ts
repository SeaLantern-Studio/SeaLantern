import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type { PluginInfo } from "@type/plugin";

interface PluginCapabilityManagerOptions {
  getPlugins: () => PluginInfo[];
}

interface PluginSettingsWriterOptions extends PluginCapabilityManagerOptions {
  applyThemeProviderSettings: (pluginId: string) => Promise<void>;
  applyThemeWidgetsProviderSettings: (pluginId: string) => Promise<void>;
}

export async function readPluginSettings(pluginId: string): Promise<Record<string, unknown>> {
  try {
    return await pluginApi.getPluginSettings(pluginId);
  } catch (errorCause) {
    pluginLogger.error("Store", `插件设置读取失败: ${pluginId}`, normalizeAppError(errorCause));
    return {};
  }
}

export function createPluginCapabilityManager(options: PluginCapabilityManagerOptions) {
  function hasCapability(pluginId: string, capability: string): boolean {
    const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
    return plugin?.manifest.capabilities?.includes(capability) ?? false;
  }

  return {
    hasCapability,
  };
}

export function createPluginSettingsWriter(options: PluginSettingsWriterOptions) {
  const capabilityManager = createPluginCapabilityManager(options);

  async function setPluginSettings(
    pluginId: string,
    settings: Record<string, unknown>,
  ): Promise<void> {
    try {
      await pluginApi.setPluginSettings(pluginId, settings);

      if (capabilityManager.hasCapability(pluginId, "theme-provider")) {
        await options.applyThemeProviderSettings(pluginId);
      }
      if (capabilityManager.hasCapability(pluginId, "theme-widgets-provider")) {
        await options.applyThemeWidgetsProviderSettings(pluginId);
      }
    } catch (error) {
      pluginLogger.error("Store", `插件设置保存失败: ${pluginId}`, error);
      throw error;
    }
  }

  return {
    setPluginSettings,
    hasCapability: capabilityManager.hasCapability,
  };
}
