import * as pluginApi from "@api/plugin";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { pluginLogger } from "@stores/plugin/pluginLogger";

export async function replayPluginContextMenuSnapshot() {
  try {
    const contextMenuSnapshot = await pluginApi.getPluginContextMenuSnapshot();
    if (contextMenuSnapshot.length === 0) {
      return;
    }

    pluginLogger.info("Snapshot", "开始回放右键菜单事件", { count: contextMenuSnapshot.length });
    const contextMenuStore = useContextMenuStore();
    for (const event of contextMenuSnapshot) {
      contextMenuStore.handleContextMenuEvent({
        action: event.action === "register" ? "register" : "unregister",
        plugin_id: event.plugin_id,
        context: event.context,
        items: event.items,
      });
    }
  } catch (error) {
    pluginLogger.error("Snapshot", "右键菜单快照回放失败", error);
  }
}
