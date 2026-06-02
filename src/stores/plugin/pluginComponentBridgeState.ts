import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PendingPluginComponentCreate } from "./pluginComponentBridgeShared";

export function createPluginComponentBridgeState() {
  const pendingComponentCreates = new Map<string, PendingPluginComponentCreate[]>();
  const pendingComponentDeletes = new Map<string, string[]>();
  const mountedComponentIds = new Map<string, Set<string>>();
  const pluginComponentSetMap = new Map<string, Array<{ componentId: string; prop: string }>>();

  function enqueueComponentCreate(pluginId: string, component: PendingPluginComponentCreate) {
    if (!pendingComponentCreates.has(pluginId)) {
      pendingComponentCreates.set(pluginId, []);
    }
    pendingComponentCreates.get(pluginId)!.push(component);
    pluginLogger.info(
      "Component",
      `已排入组件创建: ${component.component_type} (${component.component_id})`,
    );
  }

  function markComponentMounted(pluginId: string, componentId: string) {
    if (!mountedComponentIds.has(pluginId)) {
      mountedComponentIds.set(pluginId, new Set());
    }
    mountedComponentIds.get(pluginId)!.add(componentId);
  }

  function trackPluginComponentSet(pluginId: string, componentId: string, prop: string) {
    if (!pluginComponentSetMap.has(pluginId)) {
      pluginComponentSetMap.set(pluginId, []);
    }

    const list = pluginComponentSetMap.get(pluginId)!;
    if (!list.find((entry) => entry.componentId === componentId && entry.prop === prop)) {
      list.push({ componentId, prop });
    }
  }

  function consumePendingComponentCreates(pluginId: string) {
    const creates = pendingComponentCreates.get(pluginId) || [];
    pendingComponentCreates.delete(pluginId);
    for (const create of creates) {
      markComponentMounted(pluginId, create.component_id);
    }
    return creates;
  }

  function consumePendingComponentDeletes(pluginId: string): string[] {
    const deletes = pendingComponentDeletes.get(pluginId) || [];
    pendingComponentDeletes.delete(pluginId);
    return deletes;
  }

  function queuePluginComponentDeletes(pluginId: string) {
    const mountedIds = mountedComponentIds.get(pluginId);
    const componentIds = mountedIds ? Array.from(mountedIds) : [];
    if (componentIds.length > 0) {
      pendingComponentDeletes.set(pluginId, componentIds);
    }

    mountedComponentIds.delete(pluginId);
    pendingComponentCreates.delete(pluginId);
  }

  function clearPluginComponentSets(pluginId: string) {
    const sets = pluginComponentSetMap.get(pluginId) || [];
    pluginComponentSetMap.delete(pluginId);
    return sets;
  }

  return {
    enqueueComponentCreate,
    trackPluginComponentSet,
    consumePendingComponentCreates,
    consumePendingComponentDeletes,
    queuePluginComponentDeletes,
    clearPluginComponentSets,
  };
}
