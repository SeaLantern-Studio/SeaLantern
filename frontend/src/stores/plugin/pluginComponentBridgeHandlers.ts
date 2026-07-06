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
    // Component bridge permissions are checked per event instead of once at registration
    // time so that plugin trust changes take effect immediately without rebuilding the
    // registry or leaving stale handles alive across page transitions.
    const { plugin_id, action, component_id, callback_id } = event;
    const registry = useComponentRegistry();
    const canRead = options.hasComponentPermission(plugin_id, "ui.component.read");
    const canWrite = options.hasComponentPermission(plugin_id, "ui.component.write");
    const canProxy = options.hasComponentPermission(plugin_id, "ui.component.proxy");
    const canCreate = options.hasComponentPermission(plugin_id, "ui.component.create");

    async function reply(data: unknown) {
      // Callback ids are optional because fire-and-forget write/proxy operations do not need
      // a round-trip. Guarding here keeps each action branch focused on business semantics.
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
        // Page filtering stays in the registry so plugin callers cannot infer hidden hosts by
        // walking local route state themselves.
        await reply(registry.list(event.page_filter));
        return;
      }
      case "get": {
        if (!canRead) {
          pluginLogger.warn("Component", `已拒绝组件属性读取: ${plugin_id}`);
          await reply({ error: "permission denied" });
          return;
        }
        // Missing handles resolve to null instead of throwing so stale plugin references fail
        // the same way whether the component unmounted locally or never existed on this page.
        const handle = registry.get(component_id);
        await reply(handle ? handle.get(event.prop!) : null);
        return;
      }
      case "set": {
        if (!canWrite) {
          pluginLogger.warn("Component", `已拒绝组件属性写入: ${plugin_id}`);
          return;
        }
        // Write actions fan out to every live handle under the same component id because
        // next/classic shells may render mirrored instances during transitions.
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
        // Method calls stay single-handle because imperative APIs usually target one concrete
        // component instance; fanning them out would make return values and side effects ambiguous.
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
          // The callback channel includes plugin id, component id, and prop name so the
          // runtime can disambiguate multiple subscriptions without exposing raw handles.
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
        // Proxy handlers are append-only registrations ordered by priority. The bridge only
        // forwards payloads here; ownership of proxy selection stays in the component registry.
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
        // Proxy removal is intentionally idempotent so plugin shutdown can blindly unregister
        // without tracking whether the host page ever accepted the proxy in this session.
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
          // Creation stays async via the queue so host pages can decide where the component
          // actually mounts and can reject or delay work during route teardown.
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
