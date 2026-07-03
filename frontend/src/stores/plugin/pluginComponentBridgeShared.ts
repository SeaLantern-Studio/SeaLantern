import type { PluginInfo } from "@type/plugin";

export interface PluginComponentEvent {
  plugin_id: string;
  action: "get" | "set" | "call" | "on" | "proxy" | "list" | "remove_proxy" | "create";
  component_id: string;
  component_type?: string;
  props?: Record<string, any>;
  prop?: string;
  value?: any;
  method?: string;
  args?: any[];
  callback_id?: string;
  priority?: number;
  page_filter?: string;
}

export interface PendingPluginComponentCreate {
  plugin_id: string;
  component_type: string;
  component_id: string;
  props: Record<string, any>;
}

export const ALLOWED_PLUGIN_COMPONENT_TYPES = new Set(["SLProgress"]);

export const PLUGIN_COMPONENT_HOST_OWNER_PREFIX = "plugin-component";

export function getPluginComponentHostOwnerId(pluginId: string): string {
  return `${PLUGIN_COMPONENT_HOST_OWNER_PREFIX}:${pluginId}`;
}

export function hasPluginPermission(
  getPlugins: () => PluginInfo[],
  pluginId: string,
  permission: string,
): boolean {
  return (
    getPlugins()
      .find((plugin) => plugin.manifest.id === pluginId)
      ?.manifest.permissions?.includes(permission) ?? false
  );
}
