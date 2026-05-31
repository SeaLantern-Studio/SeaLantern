import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useComponentRegistry } from "@composables/useComponentRegistry";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginInfo } from "@type/plugin";

interface PluginComponentEvent {
  plugin_id: string;
  action: "get" | "set" | "call" | "on" | "proxy" | "list" | "remove_proxy" | "create";
  component_id: string;
  component_type?: string;
  props?: Record<string, any>;
  prop?: string;
  value?: any;
  method?: string;
  args?: any[];
  callback_id?: string;
  priority?: number;
  page_filter?: string;
}

export function createPluginComponentBridge(
  getPlugins: () => PluginInfo[],
  isBrowserEnv: () => boolean,
) {
  const pendingComponentCreates = new Map<
    string,
    Array<{
      component_type: string;
      component_id: string;
      props: Record<string, any>;
    }>
  >();
  const pendingComponentDeletes = new Map<string, string[]>();
  const pluginComponentSetMap = new Map<string, Array<{ componentId: string; prop: string }>>();
  let componentEventUnlisten: UnlistenFn | null = null;

  function hasComponentPerm(pluginId: string, permission: string): boolean {
    return (
      getPlugins()
        .find((plugin) => plugin.manifest.id === pluginId)
        ?.manifest.permissions?.includes(permission) ?? false
    );
  }

  async function handlePluginComponentEvent(event: PluginComponentEvent) {
    const { plugin_id, action, component_id, callback_id } = event;
    const registry = useComponentRegistry();
    const canRead = hasComponentPerm(plugin_id, "ui.component.read");
    const canWrite = hasComponentPerm(plugin_id, "ui.component.write");
    const canProxy = hasComponentPerm(plugin_id, "ui.component.proxy");
    const canCreate = hasComponentPerm(plugin_id, "ui.component.create");

    async function reply(data: unknown) {
      if (callback_id) {
        await emit(`plugin:component:callback:${callback_id}`, data);
      }
    }

    switch (action) {
      case "list": {
        if (!canRead) {
          pluginLogger.warn("Component", `已拒绝组件列表读取: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        await reply(registry.list(event.page_filter));
        break;
      }
      case "get": {
        if (!canRead) {
          pluginLogger.warn("Component", `已拒绝组件属性读取: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        const handle = registry.get(component_id);
        await reply(handle ? handle.get(event.prop!) : null);
        break;
      }
      case "set": {
        if (!canWrite) {
          pluginLogger.warn("Component", `已拒绝组件属性写入: ${plugin_id}`);
          return;
        }
        for (const { handle } of registry.getAll(component_id)) {
          handle.set(event.prop!, event.value);
        }
        trackPluginComponentSet(plugin_id, component_id, event.prop!);
        break;
      }
      case "call": {
        if (!canWrite) {
          pluginLogger.warn("Component", `已拒绝组件方法调用: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        const handle = registry.get(component_id);
        await reply(handle ? handle.call(event.method!, ...(event.args ?? [])) : null);
        break;
      }
      case "on": {
        if (!canRead) {
          pluginLogger.warn("Component", `已拒绝组件事件监听: ${plugin_id}`);
          return;
        }
        const handle = registry.get(component_id);
        if (handle && event.prop) {
          handle.on(event.prop, async (...args: any[]) => {
            await emit(`plugin:component:event:${plugin_id}:${component_id}:${event.prop}`, args);
          });
        }
        break;
      }
      case "proxy": {
        if (!canProxy) {
          pluginLogger.warn("Component", `已拒绝组件代理注册: ${plugin_id}`);
          return;
        }
        registry.addProxy(component_id, {
          pluginId: plugin_id,
          priority: event.priority ?? 0,
          handler: (payload) => {
            if (callback_id) {
              void emit(`plugin:component:proxy:${callback_id}`, payload);
            }
          },
        });
        break;
      }
      case "remove_proxy": {
        registry.removeProxy(component_id, plugin_id);
        break;
      }
      case "create": {
        if (!canCreate) {
          pluginLogger.warn("Component", `已拒绝组件创建: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        if (event.component_type && component_id) {
          if (!pendingComponentCreates.has(plugin_id)) {
            pendingComponentCreates.set(plugin_id, []);
          }
          pendingComponentCreates.get(plugin_id)!.push({
            component_type: event.component_type,
            component_id,
            props: event.props || {},
          });
          pluginLogger.info(
            "Component",
            `已排入组件创建: ${event.component_type} (${component_id})`,
          );
          await reply({ ok: true });
          return;
        }
        await reply({ error: "invalid payload" });
        break;
      }
    }
  }

  async function initComponentEventListener() {
    if (isBrowserEnv() || componentEventUnlisten) {
      return;
    }
    try {
      componentEventUnlisten = await listen<PluginComponentEvent>(
        "plugin:ui:component",
        (event) => {
          void handlePluginComponentEvent(event.payload);
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

  function trackPluginComponentSet(pluginId: string, componentId: string, prop: string) {
    if (!pluginComponentSetMap.has(pluginId)) {
      pluginComponentSetMap.set(pluginId, []);
    }
    const list = pluginComponentSetMap.get(pluginId)!;
    if (!list.find((entry) => entry.componentId === componentId && entry.prop === prop)) {
      list.push({ componentId, prop });
    }
  }

  function removePluginProxies(pluginId: string) {
    const registry = useComponentRegistry();
    for (const { id } of registry.list()) {
      registry.removeProxy(id, pluginId);
    }
    const sets = pluginComponentSetMap.get(pluginId) || [];
    for (const { componentId, prop } of sets) {
      for (const { handle } of registry.getAll(componentId)) {
        handle.set(prop, null);
      }
    }
    pluginComponentSetMap.delete(pluginId);
    pluginLogger.info("Component", `已清理组件代理: ${pluginId}`);
  }

  function removePluginComponents(pluginId: string) {
    const creates = pendingComponentCreates.get(pluginId) || [];
    const componentIds = creates.map((item) => item.component_id);
    if (componentIds.length > 0) {
      pendingComponentDeletes.set(pluginId, componentIds);
    }
    pendingComponentCreates.delete(pluginId);
  }

  function consumePendingComponentCreates(pluginId: string) {
    const creates = pendingComponentCreates.get(pluginId) || [];
    pendingComponentCreates.delete(pluginId);
    return creates;
  }

  function consumePendingComponentDeletes(pluginId: string): string[] {
    const deletes = pendingComponentDeletes.get(pluginId) || [];
    pendingComponentDeletes.delete(pluginId);
    return deletes;
  }

  return {
    handlePluginComponentEvent,
    initComponentEventListener,
    cleanupComponentEventListener,
    removePluginProxies,
    removePluginComponents,
    consumePendingComponentCreates,
    consumePendingComponentDeletes,
  };
}
