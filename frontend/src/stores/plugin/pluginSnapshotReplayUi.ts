import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginUiAction } from "@type/plugin";
import type { SnapshotReplayDependencies } from "./pluginSnapshotReplayShared";

export async function replayPluginUiSnapshot(deps: SnapshotReplayDependencies) {
  try {
    const snapshot = await pluginApi.getPluginUiSnapshot();
    if (snapshot.length === 0) {
      return;
    }

    pluginLogger.info("Snapshot", "开始回放界面事件", { count: snapshot.length });
    await snapshot.reduce<Promise<void>>(async (previousReplay, event) => {
      await previousReplay;

      await deps.handlePluginUiEvent({
        plugin_id: event.plugin_id,
        action: event.action as PluginUiAction,
        element_id: event.element_id,
        html: event.html,
      });
    }, Promise.resolve());
  } catch (error) {
    pluginLogger.error("Snapshot", "界面快照回放失败", error);
  }
}
