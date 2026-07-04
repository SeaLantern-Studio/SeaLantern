import { isPluginRuntimeUiBridgeAvailable } from "@src/services/hostCapabilities";
import { replayPluginComponentSnapshot } from "./pluginSnapshotReplayComponents";
import { replayPluginContextMenuSnapshot } from "./pluginSnapshotReplayContextMenu";
import { replayPluginPermissionLogsSnapshot } from "./pluginSnapshotReplayPermissions";
import type { SnapshotReplayDependencies } from "./pluginSnapshotReplayShared";
import { replayPluginSidebarSnapshot } from "./pluginSnapshotReplaySidebar";
import { replayPluginUiSnapshot } from "./pluginSnapshotReplayUi";

export function createPluginSnapshotReplay(deps: SnapshotReplayDependencies) {
  async function replayUiSnapshot() {
    if (!(await isPluginRuntimeUiBridgeAvailable())) {
      return;
    }

    await replayPluginUiSnapshot(deps);
    await replayPluginSidebarSnapshot(deps);
    await replayPluginComponentSnapshot(deps);
    await replayPluginContextMenuSnapshot();
    await replayPluginPermissionLogsSnapshot(deps);
  }

  return {
    replayUiSnapshot,
  };
}
