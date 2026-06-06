import * as pluginApi from "@api/plugin";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginComponentEvent } from "./pluginComponentBridge";
import type { SnapshotReplayDependencies } from "./pluginSnapshotReplayShared";

function parseComponentSnapshotPayload(payloadJson: string): PluginComponentEvent | null {
  try {
    const payload = JSON.parse(payloadJson);
    return {
      plugin_id: payload.plugin_id ?? "",
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
    };
  } catch (error) {
    pluginLogger.error("Snapshot", "组件事件内容解析失败", error);
    return null;
  }
}

export async function replayPluginComponentSnapshot(deps: SnapshotReplayDependencies) {
  try {
    const componentSnapshot = await pluginApi.getPluginComponentSnapshot();
    if (componentSnapshot.length === 0) {
      return;
    }

    pluginLogger.info("Snapshot", "开始回放组件事件", { count: componentSnapshot.length });
    await componentSnapshot.reduce<Promise<void>>(async (previousReplay, event) => {
      await previousReplay;

      const payload = parseComponentSnapshotPayload(event.payload_json);
      if (!payload) {
        return;
      }

      await deps.handlePluginComponentEvent({
        ...payload,
        plugin_id: event.plugin_id,
      });
    }, Promise.resolve());
  } catch (error) {
    pluginLogger.error("Snapshot", "组件快照回放失败", error);
  }
}
