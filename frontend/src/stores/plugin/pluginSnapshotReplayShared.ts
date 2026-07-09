import type { PluginInfo, PluginPermissionLog, PluginUiAction, SidebarItem } from "@type/plugin";
import type { PluginComponentEvent } from "./pluginComponentBridge";

export interface SnapshotReplayDependencies {
  getPlugins: () => PluginInfo[];
  getSidebarItems: () => SidebarItem[];
  setSidebarItems: (items: SidebarItem[]) => void;
  handlePluginUiEvent: (event: {
    plugin_id: string;
    action: PluginUiAction;
    element_id: string;
    html: string;
  }) => Promise<void> | void;
  handlePluginComponentEvent: (event: PluginComponentEvent) => Promise<void> | void;
  hydratePermissionLogs: (entries: Record<string, PluginPermissionLog[]>) => void;
  slicePermissionLogs: (logs: PluginPermissionLog[]) => PluginPermissionLog[];
}
