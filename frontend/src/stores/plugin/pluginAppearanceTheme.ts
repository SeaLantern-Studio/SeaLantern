import { pluginLogger } from "@stores/plugin/pluginLogger";
import { setThemeProviderOverrides } from "@utils/theme";
import {
  findPluginById,
  getPluginDefaults,
  type PluginAppearanceManagerOptions,
} from "./pluginAppearanceShared";

export const PLUGIN_THEME_CUSTOM_STYLE_ID = "plugin-theme-custom";

const THEME_ALIAS_MAP: Record<string, string> = {
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

const ROOT_THEME_VARS_TO_CLEAR = [
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

function getEnabledThemeProviderIds(
  options: Pick<PluginAppearanceManagerOptions, "getPlugins">,
  explicitPluginIds?: string[],
): string[] {
  if (explicitPluginIds) {
    return explicitPluginIds;
  }

  return options
    .getPlugins()
    .filter((plugin) => plugin.state === "enabled")
    .filter((plugin) => plugin.manifest.capabilities?.includes("theme-provider"))
    .map((plugin) => plugin.manifest.id);
}

function injectThemeStyle(
  options: Pick<PluginAppearanceManagerOptions, "sanitizeCss">,
  settings: Record<string, unknown>,
  variableMap: Record<string, string>,
  defaults: Record<string, string>,
) {
  let customCss = ":root {\n";
  let hasCustomValues = false;

  for (const [key, cssVar] of Object.entries(variableMap)) {
    const value = settings[key] ?? defaults[key];
    if (value === undefined) {
      continue;
    }

    customCss += `  ${cssVar}: ${String(value)};\n`;
    const aliasVar = THEME_ALIAS_MAP[key];
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

function clearRootThemeOverrides() {
  const root = document.documentElement;
  for (const cssVar of ROOT_THEME_VARS_TO_CLEAR) {
    root.style.removeProperty(cssVar);
  }
}

function removeThemeWidgetsSettings() {
  const root = document.documentElement;
  root.removeAttribute("data-glow-intensity");
  root.removeAttribute("data-gradient-text");
  root.removeAttribute("data-particles");
}

export function createPluginAppearanceThemeManager(options: PluginAppearanceManagerOptions) {
  function syncThemeProviderOverrides(explicitPluginIds?: string[]) {
    setThemeProviderOverrides(getEnabledThemeProviderIds(options, explicitPluginIds));
  }

  async function applyThemeProviderSettings(pluginId: string) {
    const plugin = findPluginById(options, pluginId);
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

      injectThemeStyle(options, settings, plugin.manifest.theme_var_map, getPluginDefaults(plugin));
      clearRootThemeOverrides();
    } catch (error) {
      pluginLogger.error("Appearance", `主题提供者设置应用失败: ${pluginId}`, error);
    }
  }

  function removeThemeSettings() {
    document.documentElement.removeAttribute("data-theme-preset");
    document.getElementById(PLUGIN_THEME_CUSTOM_STYLE_ID)?.remove();
  }

  async function applyThemeWidgetsProviderSettings(pluginId: string) {
    const plugin = findPluginById(options, pluginId);
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
  return {
    syncThemeProviderOverrides,
    applyThemeProviderSettings,
    removeThemeSettings,
    applyThemeWidgetsProviderSettings,
    removeThemeWidgetsSettings,
  };
}
