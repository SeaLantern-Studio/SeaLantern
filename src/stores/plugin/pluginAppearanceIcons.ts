import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginAppearanceManagerOptions } from "./pluginAppearanceShared";

export async function loadPluginIcons(options: PluginAppearanceManagerOptions) {
  await Promise.all(
    options.getPlugins().map(async (plugin) => {
      if (plugin.manifest.icon && !options.getIcons()[plugin.manifest.id]) {
        try {
          const iconData = await pluginApi.getPluginIcon(plugin.manifest.id);
          if (iconData) {
            options.setIcon(plugin.manifest.id, iconData);
          }
        } catch (error) {
          pluginLogger.error("Appearance", `插件图标读取失败: ${plugin.manifest.id}`, error);
        }
      }
    }),
  );
}
