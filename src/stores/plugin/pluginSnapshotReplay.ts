import * as pluginApi from "@api/plugin";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type {
  PluginInfo,
  PluginPermissionLog,
  PluginUiAction,
  SidebarItem,
  SidebarMode,
} from "@type/plugin";

interface SnapshotReplayDependencies {
  getPlugins: () => PluginInfo[];
  getSidebarItems: () => SidebarItem[];
  setSidebarItems: (items: SidebarItem[]) => void;
  handlePluginUiEvent: (event: {
    plugin_id: string;
    action: PluginUiAction;
    element_id: string;
    html: string;
  }) => Promise<void> | void;
  handlePluginComponentEvent: (event: {
    plugin_id: string;
    action: "get" | "set" | "call" | "on" | "proxy" | "list" | "remove_proxy" | "create";
    component_id: string;
    component_type?: string;
    props?: Record<string, any>;
    callback_id?: string;
    prop?: string;
    value?: any;
    method?: string;
    args?: any[];
    priority?: number;
    page_filter?: string;
  }) => Promise<void> | void;
  hydratePermissionLogs: (entries: Record<string, PluginPermissionLog[]>) => void;
  slicePermissionLogs: (logs: PluginPermissionLog[]) => PluginPermissionLog[];
}

export function createPluginSnapshotReplay(deps: SnapshotReplayDependencies) {
  async function replayUiSnapshot() {
    try {
      const snapshot = await pluginApi.getPluginUiSnapshot();
      if (snapshot.length > 0) {
        pluginLogger.info("Snapshot", `开始回放界面事件`, { count: snapshot.length });
        for (const event of snapshot) {
          await deps.handlePluginUiEvent({
            plugin_id: event.plugin_id,
            action: event.action as PluginUiAction,
            element_id: event.element_id,
            html: event.html,
          });
        }
      }
    } catch (error) {
      pluginLogger.error("Snapshot", "界面快照回放失败", error);
    }

    try {
      const sidebarSnapshot = await pluginApi.getPluginSidebarSnapshot();
      if (sidebarSnapshot.length > 0) {
        pluginLogger.info("Snapshot", `开始回放侧边栏事件`, { count: sidebarSnapshot.length });
        for (const event of sidebarSnapshot) {
          if (event.action !== "register") {
            continue;
          }
          const sidebarMode: SidebarMode = "self";
          const filtered = deps
            .getSidebarItems()
            .filter((item) => item.pluginId !== event.plugin_id);
          filtered.push({
            pluginId: event.plugin_id,
            label: event.label,
            icon: event.icon || undefined,
            mode: sidebarMode,
            showDependents: true,
            priority: 100,
          });
          filtered.sort((a, b) => a.priority - b.priority);
          deps.setSidebarItems(filtered);
        }
      }
    } catch (error) {
      pluginLogger.error("Snapshot", "侧边栏快照回放失败", error);
    }

    try {
      const componentSnapshot = await pluginApi.getPluginComponentSnapshot();
      if (componentSnapshot.length > 0) {
        pluginLogger.info("Snapshot", `开始回放组件事件`, { count: componentSnapshot.length });
        for (const event of componentSnapshot) {
          try {
            const payload = JSON.parse(event.payload_json);
            await deps.handlePluginComponentEvent({
              plugin_id: event.plugin_id,
              action: payload.action,
              component_id: payload.component_id,
              component_type: payload.component_type,
              props: payload.props,
              callback_id: payload.callback_id ?? "",
              prop: payload.prop,
              value: payload.value,
              method: payload.method,
              args: payload.args,
              priority: payload.priority,
              page_filter: payload.page_filter,
            });
          } catch (parseError) {
            pluginLogger.error("Snapshot", "组件事件内容解析失败", parseError);
          }
        }
      }
    } catch (error) {
      pluginLogger.error("Snapshot", "组件快照回放失败", error);
    }

    try {
      const contextMenuSnapshot = await pluginApi.getPluginContextMenuSnapshot();
      if (contextMenuSnapshot.length > 0) {
        pluginLogger.info("Snapshot", `开始回放右键菜单事件`, {
          count: contextMenuSnapshot.length,
        });
        const contextMenuStore = useContextMenuStore();
        for (const event of contextMenuSnapshot) {
          contextMenuStore.handleContextMenuEvent({
            action: event.action === "register" ? "register" : "unregister",
            plugin_id: event.plugin_id,
            context: event.context,
            items: event.items,
          });
        }
      }
    } catch (error) {
      pluginLogger.error("Snapshot", "右键菜单快照回放失败", error);
    }

    try {
      const permissionLogEntries = await Promise.all(
        deps
          .getPlugins()
          .map(
            async (plugin) =>
              [
                plugin.manifest.id,
                await pluginApi.getPluginPermissionLogs(plugin.manifest.id),
              ] as const,
          ),
      );

      const nextPermissionLogs: Record<string, PluginPermissionLog[]> = {};
      for (const [pluginId, logs] of permissionLogEntries) {
        nextPermissionLogs[pluginId] = deps.slicePermissionLogs(logs);
      }
      deps.hydratePermissionLogs(nextPermissionLogs);
    } catch (error) {
      pluginLogger.error("Snapshot", "权限记录快照回放失败", error);
    }
  }

  return {
    replayUiSnapshot,
  };
}
