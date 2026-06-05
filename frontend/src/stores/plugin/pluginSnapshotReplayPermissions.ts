import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginPermissionLog } from "@type/plugin";
import type { SnapshotReplayDependencies } from "./pluginSnapshotReplayShared";

export async function replayPluginPermissionLogsSnapshot(deps: SnapshotReplayDependencies) {
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
