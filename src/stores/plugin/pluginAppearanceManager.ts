import { createPluginAppearanceCssManager } from "./pluginAppearanceCss";
import { loadPluginIcons } from "./pluginAppearanceIcons";
import {
  createPluginAppearanceThemeManager,
  PLUGIN_THEME_CUSTOM_STYLE_ID,
} from "./pluginAppearanceTheme";
import { hasPluginCapability, type PluginAppearanceManagerOptions } from "./pluginAppearanceShared";

export { PLUGIN_THEME_CUSTOM_STYLE_ID };

export function createPluginAppearanceManager(options: PluginAppearanceManagerOptions) {
  function hasCapability(pluginId: string, capability: string): boolean {
    return hasPluginCapability(options, pluginId, capability);
  }

  const themeManager = createPluginAppearanceThemeManager(options);
  const cssManager = createPluginAppearanceCssManager(options, {
    hasCapability,
    syncThemeProviderOverrides: themeManager.syncThemeProviderOverrides,
    applyThemeProviderSettings: themeManager.applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings: themeManager.applyThemeWidgetsProviderSettings,
    removeThemeSettings: themeManager.removeThemeSettings,
    removeThemeWidgetsSettings: themeManager.removeThemeWidgetsSettings,
  });

  return {
    syncThemeProviderOverrides: themeManager.syncThemeProviderOverrides,
    loadPluginIcons: () => loadPluginIcons(options),
    injectPluginCss: cssManager.injectPluginCss,
    removePluginCss: cssManager.removePluginCss,
    injectAllPluginCss: cssManager.injectAllPluginCss,
    applyThemeProviderSettings: themeManager.applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings: themeManager.applyThemeWidgetsProviderSettings,
  };
}
