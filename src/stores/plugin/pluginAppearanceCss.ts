import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { findPluginById, type PluginAppearanceManagerOptions } from "./pluginAppearanceShared";

interface PluginAppearanceCssManagerDependencies {
  hasCapability: (pluginId: string, capability: string) => boolean;
  syncThemeProviderOverrides: (explicitPluginIds?: string[]) => void;
  applyThemeProviderSettings: (pluginId: string) => Promise<void>;
  applyThemeWidgetsProviderSettings: (pluginId: string) => Promise<void>;
  removeThemeSettings: () => void;
  removeThemeWidgetsSettings: () => void;
}

function createPluginStyleElement(pluginId: string, css: string) {
  const style = document.createElement("style");
  style.id = `plugin-manifest-css-${pluginId}`;
  style.setAttribute("data-plugin-id", pluginId);
  style.setAttribute("data-plugin-source", "manifest");
  style.textContent = css;
  return style;
}

export function createPluginAppearanceCssManager(
  options: PluginAppearanceManagerOptions,
  dependencies: PluginAppearanceCssManagerDependencies,
) {
  async function injectPluginCss(pluginId: string) {
    try {
      const css = await pluginApi.getPluginCss(pluginId);
      if (!css) {
        return;
      }

      removePluginCss(pluginId);
      document.head.appendChild(createPluginStyleElement(pluginId, options.sanitizeCss(css)));

      if (dependencies.hasCapability(pluginId, "theme-provider")) {
        await dependencies.applyThemeProviderSettings(pluginId);
      }
      if (dependencies.hasCapability(pluginId, "theme-widgets-provider")) {
        await dependencies.applyThemeWidgetsProviderSettings(pluginId);
      }
    } catch (error) {
      pluginLogger.error("Appearance", `插件样式注入失败: ${pluginId}`, error);
    }
  }

  function removePluginCss(pluginId: string) {
    document.querySelectorAll(`style[data-plugin-id="${pluginId}"]`).forEach((element) => {
      element.remove();
    });

    if (dependencies.hasCapability(pluginId, "theme-provider")) {
      dependencies.removeThemeSettings();
    }
    if (dependencies.hasCapability(pluginId, "theme-widgets-provider")) {
      dependencies.removeThemeWidgetsSettings();
    }
  }

  async function injectAllPluginCss() {
    try {
      const allCss = await pluginApi.getAllPluginCss();

      dependencies.removeThemeSettings();
      dependencies.removeThemeWidgetsSettings();
      dependencies.syncThemeProviderOverrides();

      document.querySelectorAll("style[data-plugin-id]").forEach((element) => {
        const id = (element as HTMLElement).id || "";
        if (id.startsWith("plugin-css-")) {
          return;
        }
        element.remove();
      });

      for (const [pluginId, css] of allCss) {
        const plugin = findPluginById(options, pluginId);
        if (!plugin?.manifest.permissions?.includes("ui") || !css) {
          continue;
        }

        document.head.appendChild(createPluginStyleElement(pluginId, options.sanitizeCss(css)));
      }

      const providerUpdates: Promise<void>[] = [];
      for (const [pluginId] of allCss) {
        if (dependencies.hasCapability(pluginId, "theme-provider")) {
          providerUpdates.push(dependencies.applyThemeProviderSettings(pluginId));
        }
        if (dependencies.hasCapability(pluginId, "theme-widgets-provider")) {
          providerUpdates.push(dependencies.applyThemeWidgetsProviderSettings(pluginId));
        }
      }

      await Promise.all(providerUpdates);
    } catch (error) {
      pluginLogger.error("Appearance", "插件样式批量注入失败", error);
    }
  }

  return {
    injectPluginCss,
    removePluginCss,
    injectAllPluginCss,
  };
}
