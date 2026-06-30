import type { InjectionKey, ShallowRef } from "vue";
import { inject, provide, shallowRef, watch, onBeforeUnmount, onMounted } from "vue";
import { useRoute } from "vue-router";
import { getPluginSidebarSnapshot, type BufferedSidebarEvent } from "@api/plugin";
import { notifyPluginPageLifecycle } from "@router/pluginPageLifecycle";

export type NextHostLifecycleEventName = "shell.ready" | "route.enter" | "route.leave";

export interface NextHostLifecyclePayload {
  route_key: string;
  page_kind: string;
  shell_id: "next";
  path: string;
}

export interface NextSidebarHostItem {
  pluginId: string;
  label: string;
  icon?: string;
}

export interface NextHostRuntimeContext {
  sidebarItems: ShallowRef<NextSidebarHostItem[]>;
  lifecycleEvent: ShallowRef<{
    name: NextHostLifecycleEventName;
    payload: NextHostLifecyclePayload;
  } | null>;
}

const NEXT_HOST_RUNTIME_KEY: InjectionKey<NextHostRuntimeContext> = Symbol("next-host-runtime");

function getPageKind(route: ReturnType<typeof useRoute>): string {
  return typeof route.meta.pageKind === "string" ? route.meta.pageKind : "unknown";
}

function dispatchLifecycleEvent(
  name: NextHostLifecycleEventName,
  payload: NextHostLifecyclePayload,
  lifecycleEvent: ShallowRef<{
    name: NextHostLifecycleEventName;
    payload: NextHostLifecyclePayload;
  } | null>,
): void {
  lifecycleEvent.value = { name, payload };

  window.dispatchEvent(
    new CustomEvent(`sealantern:next-host:${name}`, {
      detail: payload,
    }),
  );
}

function buildSidebarItems(snapshot: BufferedSidebarEvent[]): NextSidebarHostItem[] {
  const items = new Map<string, NextSidebarHostItem>();

  for (const event of snapshot) {
    if (event.action === "unregister") {
      items.delete(event.plugin_id);
      continue;
    }

    if (event.action !== "register") {
      continue;
    }

    items.set(event.plugin_id, {
      pluginId: event.plugin_id,
      label: event.label,
      icon: event.icon || undefined,
    });
  }

  return Array.from(items.values()).sort((a, b) => a.label.localeCompare(b.label));
}

export function provideNextHostRuntime(): NextHostRuntimeContext {
  const route = useRoute();
  const sidebarItems = shallowRef<NextSidebarHostItem[]>([]);
  const lifecycleEvent = shallowRef<{
    name: NextHostLifecycleEventName;
    payload: NextHostLifecyclePayload;
  } | null>(null);

  let routeSequence = 0;
  let currentPayload: NextHostLifecyclePayload | null = null;

  function buildPayload(): NextHostLifecyclePayload {
    return {
      route_key: `next:${routeSequence}`,
      page_kind: getPageKind(route),
      shell_id: "next",
      path: route.fullPath,
    };
  }

  async function hydrateSidebarSnapshot(): Promise<void> {
    try {
      const snapshot = await getPluginSidebarSnapshot();
      sidebarItems.value = buildSidebarItems(snapshot);
    } catch {
      sidebarItems.value = [];
    }
  }

  onMounted(async () => {
    routeSequence += 1;
    currentPayload = buildPayload();

    await hydrateSidebarSnapshot();
    dispatchLifecycleEvent("shell.ready", currentPayload, lifecycleEvent);
    notifyPluginPageLifecycle(route.fullPath);
    dispatchLifecycleEvent("route.enter", currentPayload, lifecycleEvent);
  });

  watch(
    () => route.fullPath,
    (nextPath, previousPath) => {
      if (!currentPayload || nextPath === previousPath) {
        return;
      }

      dispatchLifecycleEvent("route.leave", currentPayload, lifecycleEvent);
      routeSequence += 1;
      currentPayload = buildPayload();
      notifyPluginPageLifecycle(nextPath);
      dispatchLifecycleEvent("route.enter", currentPayload, lifecycleEvent);
    },
  );

  onBeforeUnmount(() => {
    if (!currentPayload) {
      return;
    }

    dispatchLifecycleEvent("route.leave", currentPayload, lifecycleEvent);
  });

  const context: NextHostRuntimeContext = {
    sidebarItems,
    lifecycleEvent,
  };

  provide(NEXT_HOST_RUNTIME_KEY, context);
  return context;
}

export function useNextHostRuntime(): NextHostRuntimeContext {
  const context = inject(NEXT_HOST_RUNTIME_KEY);
  if (!context) {
    throw new Error("Next host runtime is not provided");
  }

  return context;
}
