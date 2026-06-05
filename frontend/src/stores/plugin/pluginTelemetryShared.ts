import type { PluginInfo } from "@type/plugin";

const HIGH_RISK_PERMISSIONS = new Set([
  "network",
  "fs",
  "server",
  "console",
  "element",
  "system",
  "execute_program",
  "plugin_folder_access",
]);

export function getPluginHighRiskPermissions(
  getPlugins: () => PluginInfo[],
  pluginId: string,
): string[] {
  const plugin = getPlugins().find((item) => item.manifest.id === pluginId);
  if (!plugin?.manifest.permissions) {
    return [];
  }

  return plugin.manifest.permissions.filter((permission) => HIGH_RISK_PERMISSIONS.has(permission));
}
