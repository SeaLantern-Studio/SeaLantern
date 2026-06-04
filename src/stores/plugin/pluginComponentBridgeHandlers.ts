import { emit } from "@tauri-apps/api/event";
import { useComponentRegistry } from "@composables/useComponentRegistry";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  ALLOWED_PLUGIN_COMPONENT_TYPES,
  type PluginComponentEvent,
  type PendingPluginComponentCreate,
} from "./pluginComponentBridgeShared";

interface PluginComponentBridgeHandlersOptions {
  hasComponentPermission: (pluginId: string, permission: string) => boolean;
  enqueueComponentCreate: (pluginId: string, component: PendingPluginComponentCreate) => void;
  trackPluginComponentSet: (pluginId: string, componentId: string, prop: string) => void;
}

export function createPluginComponentBridgeHandlers(options: PluginComponentBridgeHandlersOptions) {
  async function handlePluginComponentEvent(event: PluginComponentEvent) {
    const { plugin_id, action, component_id, callback_id } = event;
    const registry = useComponentRegistry();
    const canRead = options.hasComponentPermission(plugin_id, "ui.component.read");
    const canWrite = options.hasComponentPermission(plugin_id, "ui.component.write");
    const canProxy = options.hasComponentPermission(plugin_id, "ui.component.proxy");
    const canCreate = options.hasComponentPermission(plugin_id, "ui.component.create");

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
        return;
      }
      case "get": {
        if (!canRead) {
          pluginLogger.warn("Component", `已拒绝组件属性读取: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        const handle = registry.get(component_id);
        await reply(handle ? handle.get(event.prop!) : null);
        return;
      }
      case "set": {
        if (!canWrite) {
          pluginLogger.warn("Component", `已拒绝组件属性写入: ${plugin_id}`);
          return;
        }
        for (const { handle } of registry.getAll(component_id)) {
          handle.set(event.prop!, event.value);
        }
        options.trackPluginComponentSet(plugin_id, component_id, event.prop!);
        return;
      }
      case "call": {
        if (!canWrite) {
          pluginLogger.warn("Component", `已拒绝组件方法调用: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        const handle = registry.get(component_id);
        await reply(handle ? handle.call(event.method!, ...(event.args ?? [])) : null);
        return;
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
        return;
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
        return;
      }
      case "remove_proxy": {
        registry.removeProxy(component_id, plugin_id);
        return;
      }
      case "create": {
        if (!canCreate) {
          pluginLogger.warn("Component", `已拒绝组件创建: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        if (event.component_type && component_id) {
          if (!ALLOWED_PLUGIN_COMPONENT_TYPES.has(event.component_type)) {
            pluginLogger.warn("Component", `已拦截未开放组件类型: ${plugin_id}`, {
              componentType: event.component_type,
            });
            await reply({ error: "component type not allowed" });
            return;
          }
          options.enqueueComponentCreate(plugin_id, {
            plugin_id,
            component_type: event.component_type,
            component_id,
            props: event.props || {},
          });
          await reply({ ok: true });
          return;
        }
        await reply({ error: "invalid payload" });
        return;
      }
    }
  }

  return {
    handlePluginComponentEvent,
  };
}
