import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { setThemeProviderOverrides } from "@utils/theme";
import type { PluginInfo } from "@type/plugin";

export const PLUGIN_THEME_CUSTOM_STYLE_ID = "plugin-theme-custom";

interface PluginAppearanceManagerOptions {
  getPlugins: () => PluginInfo[];
  getIcons: () => Record<string, string>;
  setIcon: (pluginId: string, iconData: string) => void;
  sanitizeCss: (css: string) => string;
  getPluginSettings: (pluginId: string) => Promise<Record<string, unknown>>;
}

export function createPluginAppearanceManager(options: PluginAppearanceManagerOptions) {
  function hasCapability(pluginId: string, capability: string): boolean {
    const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
    return plugin?.manifest.capabilities?.includes(capability) ?? false;
  }

  function getEnabledThemeProviderIds(explicitPluginIds?: string[]): string[] {
    if (explicitPluginIds) {
      return explicitPluginIds;
    }

    return options
      .getPlugins()
      .filter((plugin) => plugin.state === "enabled")
      .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
      .map((plugin) => plugin.manifest.id);
  }

  function syncThemeProviderOverrides(explicitPluginIds?: string[]) {
    setThemeProviderOverrides(getEnabledThemeProviderIds(explicitPluginIds));
  }

  async function loadPluginIcons() {
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

  async function injectPluginCss(pluginId: string) {
    try {
      const css = await pluginApi.getPluginCss(pluginId);
      if (!css) {
        return;
      }

      removePluginCss(pluginId);
      const style = document.createElement("style");
      style.id = `plugin-manifest-css-${pluginId}`;
      style.setAttribute("data-plugin-id", pluginId);
      style.setAttribute("data-plugin-source", "manifest");
      style.textContent = options.sanitizeCss(css);
      document.head.appendChild(style);

      if (hasCapability(pluginId, "theme-provider")) {
        await applyThemeProviderSettings(pluginId);
      }
      if (hasCapability(pluginId, "theme-widgets-provider")) {
        await applyThemeWidgetsProviderSettings(pluginId);
      }
    } catch (error) {
      pluginLogger.error("Appearance", `插件样式注入失败: ${pluginId}`, error);
    }
  }

  function removePluginCss(pluginId: string) {
    document
      .querySelectorAll(`style[data-plugin-id="${pluginId}"]`)
      .forEach((element) => element.remove());

    if (hasCapability(pluginId, "theme-provider")) {
      removeThemeSettings();
    }
    if (hasCapability(pluginId, "theme-widgets-provider")) {
      removeThemeWidgetsSettings();
    }
  }

  async function injectAllPluginCss() {
    try {
      const allCss = await pluginApi.getAllPluginCss();

      removeThemeSettings();
      removeThemeWidgetsSettings();
      syncThemeProviderOverrides();

      document.querySelectorAll("style[data-plugin-id]").forEach((element) => {
        const id = (element as HTMLElement).id || "";
        if (id.startsWith("plugin-css-")) {
          return;
        }
        element.remove();
      });

      for (const [pluginId, css] of allCss) {
        const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
        if (!plugin?.manifest.permissions?.includes("ui") || !css) {
          continue;
        }

        const style = document.createElement("style");
        style.id = `plugin-manifest-css-${pluginId}`;
        style.setAttribute("data-plugin-id", pluginId);
        style.setAttribute("data-plugin-source", "manifest");
        style.textContent = options.sanitizeCss(css);
        document.head.appendChild(style);
      }

      const providerUpdates: Promise<void>[] = [];
      for (const [pluginId] of allCss) {
        if (hasCapability(pluginId, "theme-provider")) {
          providerUpdates.push(applyThemeProviderSettings(pluginId));
        }
        if (hasCapability(pluginId, "theme-widgets-provider")) {
          providerUpdates.push(applyThemeWidgetsProviderSettings(pluginId));
        }
      }
      await Promise.all(providerUpdates);
    } catch (error) {
      pluginLogger.error("Appearance", "插件样式批量注入失败", error);
    }
  }

  function getPluginDefaults(pluginId: string): Record<string, string> {
    const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
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

  function injectThemeStyle(
    settings: Record<string, unknown>,
    variableMap: Record<string, string>,
    defaults: Record<string, string>,
  ) {
    const aliasMap: Record<string, string> = {
      bg_primary: "--bg-primary",
      bg_secondary: "--bg-secondary",
      bg_tertiary: "--bg-tertiary",
      accent_primary: "--accent-primary",
      accent_secondary: "--accent-secondary",
      text_primary: "--text-primary",
      text_secondary: "--text-secondary",
      border_color: "--border-color",
      glass_blur: "--sl-glass-blur",
      border_radius: "--sl-radius-lg",
    };

    let customCss = ":root {\n";
    let hasCustomValues = false;
    for (const [key, cssVar] of Object.entries(variableMap)) {
      const value = settings[key] ?? defaults[key];
      if (value === undefined) {
        continue;
      }

      customCss += `  ${cssVar}: ${String(value)};\n`;
      const aliasVar = aliasMap[key];
      if (aliasVar) {
        customCss += `  ${aliasVar}: ${String(value)};\n`;
      }
      hasCustomValues = true;

      if (key === "border_radius") {
        const radiusValue = parseFloat(String(value));
        if (!Number.isNaN(radiusValue)) {
          customCss += `  --sl-radius-md: ${radiusValue * 0.625}px;\n`;
          customCss += `  --sl-radius-sm: ${radiusValue * 0.375}px;\n`;
          customCss += `  --sl-radius-xl: ${radiusValue * 1.5}px;\n`;
          customCss += `  --radius-md: ${radiusValue * 0.625}px;\n`;
          customCss += `  --radius-sm: ${radiusValue * 0.375}px;\n`;
        }
      }
    }
    customCss += "}\n";

    let customStyle = document.getElementById(
      PLUGIN_THEME_CUSTOM_STYLE_ID,
    ) as HTMLStyleElement | null;
    if (hasCustomValues) {
      if (!customStyle) {
        customStyle = document.createElement("style");
        customStyle.id = PLUGIN_THEME_CUSTOM_STYLE_ID;
        document.head.appendChild(customStyle);
      }
      customStyle.textContent = options.sanitizeCss(customCss);
      return;
    }

    customStyle?.remove();
  }

  async function applyThemeProviderSettings(pluginId: string) {
    const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
    if (!plugin?.manifest.theme_var_map) {
      return;
    }

    try {
      const settings = await options.getPluginSettings(pluginId);
      const preset = settings.preset;
      if (preset && preset !== "default") {
        document.documentElement.setAttribute("data-theme-preset", String(preset));
      } else {
        document.documentElement.removeAttribute("data-theme-preset");
      }

      injectThemeStyle(settings, plugin.manifest.theme_var_map, getPluginDefaults(pluginId));

      const root = document.documentElement;
      const varsToClear = [
        "--sl-bg",
        "--sl-bg-secondary",
        "--sl-bg-tertiary",
        "--sl-primary",
        "--sl-primary-light",
        "--sl-primary-dark",
        "--sl-primary-bg",
        "--sl-accent",
        "--sl-accent-light",
        "--sl-text-primary",
        "--sl-text-secondary",
        "--sl-text-tertiary",
        "--sl-border",
        "--sl-border-light",
        "--sl-surface",
        "--sl-surface-hover",
        "--sl-shadow-sm",
        "--sl-shadow-md",
        "--sl-shadow-lg",
        "--sl-shadow-xl",
      ];
      for (const cssVar of varsToClear) {
        root.style.removeProperty(cssVar);
      }
    } catch (error) {
      pluginLogger.error("Appearance", `主题提供者设置应用失败: ${pluginId}`, error);
    }
  }

  function removeThemeSettings() {
    document.documentElement.removeAttribute("data-theme-preset");
    document.getElementById(PLUGIN_THEME_CUSTOM_STYLE_ID)?.remove();
  }

  async function applyThemeWidgetsProviderSettings(pluginId: string) {
    const plugin = options.getPlugins().find((item) => item.manifest.id === pluginId);
    if (!plugin) {
      return;
    }

    try {
      const settings = await options.getPluginSettings(pluginId);
      const root = document.documentElement;

      const booleanKeys =
        plugin.manifest.settings
          ?.filter((setting) => setting.type === "boolean")
          .map((setting) => setting.key) ?? [];
      for (const key of booleanKeys) {
        const enabled = settings[key] === true || settings[key] === "true";
        root.classList.toggle(`plugin-${key.replace(/_/g, "-")}`, enabled);
      }

      const numberKeys =
        plugin.manifest.settings
          ?.filter((setting) => setting.type === "number")
          .map((setting) => setting.key) ?? [];
      for (const key of numberKeys) {
        const value = settings[key];
        if (value !== undefined) {
          root.style.setProperty(`--plugin-${key.replace(/_/g, "-")}`, String(value));
        }
      }
    } catch (error) {
      pluginLogger.error("Appearance", `主题挂件设置应用失败: ${pluginId}`, error);
    }
  }

  function removeThemeWidgetsSettings() {
    const root = document.documentElement;
    root.removeAttribute("data-glow-intensity");
    root.removeAttribute("data-gradient-text");
    root.removeAttribute("data-particles");
  }

  return {
    syncThemeProviderOverrides,
    loadPluginIcons,
    injectPluginCss,
    removePluginCss,
    injectAllPluginCss,
    applyThemeProviderSettings,
    applyThemeWidgetsProviderSettings,
  };
}
