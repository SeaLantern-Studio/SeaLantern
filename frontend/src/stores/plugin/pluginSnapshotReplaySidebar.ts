import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { SidebarMode } from "@type/plugin";
import type { SnapshotReplayDependencies } from "./pluginSnapshotReplayShared";

export async function replayPluginSidebarSnapshot(deps: SnapshotReplayDependencies) {
  try {
    const sidebarSnapshot = await pluginApi.getPluginSidebarSnapshot();
    if (sidebarSnapshot.length === 0) {
      return;
    }

    pluginLogger.info("Snapshot", "开始回放侧边栏事件", { count: sidebarSnapshot.length });
    for (const event of sidebarSnapshot) {
      if (event.action !== "register") {
        continue;
      }

      const sidebarMode: SidebarMode = "self";
      const filtered = deps.getSidebarItems().filter((item) => item.pluginId !== event.plugin_id);
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
  } catch (error) {
    pluginLogger.error("Snapshot", "侧边栏快照回放失败", error);
  }
}
