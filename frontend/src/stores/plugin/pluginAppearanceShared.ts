import type { PluginInfo } from "@type/plugin";

export interface PluginAppearanceManagerOptions {
  getPlugins: () => PluginInfo[];
  getIcons: () => Record<string, string>;
  setIcon: (pluginId: string, iconData: string) => void;
  sanitizeCss: (css: string) => string;
  getPluginSettings: (pluginId: string) => Promise<Record<string, unknown>>;
}

export function findPluginById(
  options: Pick<PluginAppearanceManagerOptions, "getPlugins">,
  pluginId: string,
): PluginInfo | undefined {
  return options.getPlugins().find((plugin) => plugin.manifest.id === pluginId);
}

export function hasPluginCapability(
  options: Pick<PluginAppearanceManagerOptions, "getPlugins">,
  pluginId: string,
  capability: string,
): boolean {
  return findPluginById(options, pluginId)?.manifest.capabilities?.includes(capability) ?? false;
}

export function getPluginDefaults(plugin: PluginInfo | undefined): Record<string, string> {
  if (!plugin?.manifest.settings) {
    return {};
  }

  const defaults: Record<string, string> = {};
  for (const field of plugin.manifest.settings) {
    if (field.default !== undefined) {
      defaults[field.key] = String(field.default);
    }
  }
  return defaults;
}
