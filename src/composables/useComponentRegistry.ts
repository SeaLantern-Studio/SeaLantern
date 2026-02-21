import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ComponentHandle {
  type: string;
  get: (prop: string) => any;
  set: (prop: string, value: any) => void;
  call: (method: string, ...args: any[]) => any;
  on: (event: string, cb: (...args: any[]) => void) => () => void;
  el: () => HTMLElement | null;
}

export interface ProxyInterceptor {
  pluginId: string;
  priority: number;
  handler: (event: { type: string; value?: any; componentId: string }) => boolean | void;
}

const registry = reactive(new Map<string, ComponentHandle>());
const interceptors = new Map<string, ProxyInterceptor[]>();

const mirrorQueue = new Map<string, Promise<unknown>>();

function enqueueMirrorOp(id: string, op: () => Promise<unknown>) {
  const prev = mirrorQueue.get(id) ?? Promise.resolve();
  let next: Promise<unknown>;
  next = prev
    .catch(() => {})
    .then(op)
    .catch(() => {})
    .finally(() => {
      if (mirrorQueue.get(id) === next) mirrorQueue.delete(id);
    });
  mirrorQueue.set(id, next);
}

export function useComponentRegistry() {
  function register(id: string, handle: ComponentHandle) {
    registry.set(id, handle);
    enqueueMirrorOp(id, () =>
      invoke("component_mirror_register", { id, componentType: handle.type }),
    );
  }

  function unregister(id: string) {
    registry.delete(id);
    interceptors.delete(id);
    enqueueMirrorOp(id, () => invoke("component_mirror_unregister", { id }));
  }

  function get(id: string): ComponentHandle | undefined {
    if (id.includes("*")) {
      const pattern = new RegExp("^" + id.replace(/\*/g, ".*") + "$");
      for (const [key, handle] of registry) {
        if (pattern.test(key)) return handle;
      }
      return undefined;
    }
    return registry.get(id);
  }

  function getAll(id: string): Array<{ id: string; handle: ComponentHandle }> {
    if (!id.includes("*")) {
      const handle = registry.get(id);
      return handle ? [{ id, handle }] : [];
    }
    const pattern = new RegExp("^" + id.replace(/\*/g, ".*") + "$");
    const result: Array<{ id: string; handle: ComponentHandle }> = [];
    for (const [key, handle] of registry) {
      if (pattern.test(key)) result.push({ id: key, handle });
    }
    return result;
  }

  function list(pageFilter?: string): Array<{ id: string; type: string }> {
    const result: Array<{ id: string; type: string }> = [];
    for (const [id, handle] of registry) {
      if (!pageFilter || id.startsWith(pageFilter + "/")) {
        result.push({ id, type: handle.type });
      }
    }
    return result;
  }

  function addProxy(componentId: string, interceptor: ProxyInterceptor) {
    if (!interceptors.has(componentId)) interceptors.set(componentId, []);
    interceptors.get(componentId)!.push(interceptor);
    interceptors.get(componentId)!.sort((a, b) => b.priority - a.priority);
  }

  function removeProxy(componentId: string, pluginId: string) {
    const list = interceptors.get(componentId);
    if (list) {
      interceptors.set(
        componentId,
        list.filter((i) => i.pluginId !== pluginId),
      );
    }
  }

  function runInterceptors(componentId: string, event: object): boolean {
    const list = interceptors.get(componentId) ?? [];
    for (const interceptor of list) {
      if (interceptor.handler(event as any) === false) return false;
    }
    return true;
  }

  return {
    register,
    unregister,
    get,
    getAll,
    list,
    addProxy,
    removeProxy,
    runInterceptors,
  };
}
