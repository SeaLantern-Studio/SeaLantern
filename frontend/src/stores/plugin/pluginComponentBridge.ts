import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useComponentRegistry } from "@composables/useComponentRegistry";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginInfo } from "@type/plugin";
import { createPluginComponentBridgeHandlers } from "./pluginComponentBridgeHandlers";
import { createPluginComponentBridgeState } from "./pluginComponentBridgeState";
import { hasPluginPermission, type PluginComponentEvent } from "./pluginComponentBridgeShared";

export type {
  PendingPluginComponentCreate,
  PluginComponentEvent,
} from "./pluginComponentBridgeShared";

export function createPluginComponentBridge(
  getPlugins: () => PluginInfo[],
  isBrowserEnv: () => boolean,
) {
  const state = createPluginComponentBridgeState();
  const handlers = createPluginComponentBridgeHandlers({
    hasComponentPermission: (pluginId, permission) =>
      hasPluginPermission(getPlugins, pluginId, permission),
    enqueueComponentCreate: state.enqueueComponentCreate,
    trackPluginComponentSet: state.trackPluginComponentSet,
  });

  let componentEventUnlisten: UnlistenFn | null = null;

  async function initComponentEventListener() {
    if (isBrowserEnv() || componentEventUnlisten) {
      return;
    }

    try {
      componentEventUnlisten = await listen<PluginComponentEvent>(
        "plugin:ui:component",
        (event) => {
          void handlers.handlePluginComponentEvent(event.payload);
        },
      );
      pluginLogger.info("Component", "组件桥接监听已就绪");
    } catch (error) {
      pluginLogger.error("Component", "组件桥接监听初始化失败", error);
    }
  }

  function cleanupComponentEventListener() {
    componentEventUnlisten?.();
    componentEventUnlisten = null;
  }

  function removePluginProxies(pluginId: string) {
    const registry = useComponentRegistry();
    for (const { id } of registry.list()) {
      registry.removeProxy(id, pluginId);
    }

    const sets = state.clearPluginComponentSets(pluginId);
    for (const { componentId, prop } of sets) {
      for (const { handle } of registry.getAll(componentId)) {
        handle.set(prop, null);
      }
    }

    pluginLogger.info("Component", `已清理组件代理: ${pluginId}`);
  }

  function removePluginComponents(pluginId: string) {
    state.queuePluginComponentDeletes(pluginId);
  }

  return {
    handlePluginComponentEvent: handlers.handlePluginComponentEvent,
    initComponentEventListener,
    cleanupComponentEventListener,
    removePluginProxies,
    removePluginComponents,
    pendingComponentVersion: state.pendingComponentVersion,
    consumePendingComponentCreates: state.consumePendingComponentCreates,
    consumePendingComponentDeletes: state.consumePendingComponentDeletes,
  };
}
