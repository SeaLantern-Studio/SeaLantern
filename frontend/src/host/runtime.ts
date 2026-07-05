import type { Component, InjectionKey, ShallowRef } from "vue";
import { inject, provide, shallowRef, watch, onBeforeUnmount, onMounted } from "vue";
import { useRoute } from "vue-router";
import { getPluginSidebarSnapshot, type BufferedSidebarEvent } from "@api/plugin";
import { notifyPluginPageLifecycle } from "@router/pluginPageLifecycle";
import { isPluginRuntimeUiBridgeAvailable } from "@src/services/hostCapabilities";
import type {
  NextHomeCardDefinitionRegistry,
  NextHomeCardRendererRegistry,
} from "../pages/home/cardRendererContract";
import { resolveNextPageKind, type NextShellPageKind } from "../contracts/page";
import {
  getNextHostSlotScope,
  isNextPageScopedHostSlot,
  type NextHostSlotId,
} from "../contracts/slots";
import {
  isNextHomePluginCardKind,
  type NextHomeHostCardDefinition,
} from "../pages/home/layoutContract";

export type NextHostLifecycleEventName = "shell.ready" | "route.enter" | "route.leave";

export interface NextHostLifecyclePayload {
  route_key: string;
  page_kind: NextShellPageKind;
  shell_id: "next";
  path: string;
}

export interface NextSidebarHostItem {
  pluginId: string;
  label: string;
  icon?: string;
}

export interface NextHostControlledComponentDefinition {
  component: Component;
  defaultSlotId: NextHostSlotId;
  allowedSlotIds?: readonly NextHostSlotId[];
  defaultOrder?: number;
}

export interface NextHostSlotRegistration {
  registrationId: string;
  ownerId: string;
  entryId: string;
  slotId: NextHostSlotId;
  scope: ReturnType<typeof getNextHostSlotScope>;
  order: number;
  component: Component;
  props: Record<string, unknown>;
  routeKey: string | null;
}

export interface NextHostSlotRegistrationInput {
  ownerId: string;
  entryId: string;
  slotId: NextHostSlotId;
  order?: number;
  component: Component;
  props?: Record<string, unknown>;
}

export interface NextHostControlledComponentRegistrationInput {
  ownerId: string;
  entryId: string;
  componentType: string;
  slotId?: NextHostSlotId;
  order?: number;
  props?: Record<string, unknown>;
}

export interface NextHostSlotRegistrationResult {
  ok: boolean;
  registrationId?: string;
  replaced?: boolean;
  conflictWith?: string;
  reason?: "unknown-component" | "slot-not-allowed" | "duplicate-entry-id";
}

interface NextHostSlotManager {
  registrations: ShallowRef<NextHostSlotRegistration[]>;
  register: (input: NextHostSlotRegistrationInput) => NextHostSlotRegistrationResult;
  unregister: (registrationId: string) => boolean;
  unregisterEntry: (ownerId: string, entryId: string) => boolean;
  registerControlledComponent: (
    input: NextHostControlledComponentRegistrationInput,
  ) => NextHostSlotRegistrationResult;
  unregisterOwner: (ownerId: string) => void;
  clearPageScopedRegistrations: (routeKey: string) => void;
}

export interface NextHostRuntimeContext {
  sidebarItems: ShallowRef<NextSidebarHostItem[]>;
  slots: NextHostSlotManager;
  home: {
    cardDefinitions: ShallowRef<NextHomeCardDefinitionRegistry>;
    cardRenderers: ShallowRef<NextHomeCardRendererRegistry>;
    registerCards: (definitions: readonly NextHomeHostCardDefinition[]) => void;
    registerCardRenderers: (registry: NextHomeCardRendererRegistry) => void;
  };
  lifecycleEvent: ShallowRef<{
    name: NextHostLifecycleEventName;
    payload: NextHostLifecyclePayload;
  } | null>;
}

const NEXT_HOST_RUNTIME_KEY: InjectionKey<NextHostRuntimeContext> = Symbol("next-host-runtime");

const CONTROLLED_COMPONENT_REGISTRY: Record<string, NextHostControlledComponentDefinition> = {};

function sortHostRegistrations(
  registrations: readonly NextHostSlotRegistration[],
): NextHostSlotRegistration[] {
  return [...registrations].toSorted((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order;
    }

    if (left.slotId !== right.slotId) {
      return left.slotId.localeCompare(right.slotId);
    }

    return left.registrationId.localeCompare(right.registrationId);
  });
}

function createNextHostSlotManager(getCurrentRouteKey: () => string | null): NextHostSlotManager {
  const registrations = shallowRef<NextHostSlotRegistration[]>([]);
  let registrationSequence = 0;

  function updateRegistrations(
    updater: (current: NextHostSlotRegistration[]) => NextHostSlotRegistration[],
  ): void {
    registrations.value = sortHostRegistrations(updater(registrations.value));
  }

  function buildRegistrationId(ownerId: string, entryId: string): string {
    registrationSequence += 1;
    return `${ownerId}:${entryId}:${registrationSequence}`;
  }

  function register(input: NextHostSlotRegistrationInput): NextHostSlotRegistrationResult {
    const scope = getNextHostSlotScope(input.slotId);
    const routeKey = isNextPageScopedHostSlot(input.slotId) ? getCurrentRouteKey() : null;
    const sameOwnerEntry = registrations.value.find(
      (entry) => entry.ownerId === input.ownerId && entry.entryId === input.entryId,
    );
    const conflictingEntry = registrations.value.find(
      (entry) => entry.entryId === input.entryId && entry.ownerId !== input.ownerId,
    );

    if (conflictingEntry) {
      return {
        ok: false,
        conflictWith: conflictingEntry.registrationId,
        reason: "duplicate-entry-id",
      };
    }

    const registrationId =
      sameOwnerEntry?.registrationId ?? buildRegistrationId(input.ownerId, input.entryId);

    const nextEntry: NextHostSlotRegistration = {
      registrationId,
      ownerId: input.ownerId,
      entryId: input.entryId,
      slotId: input.slotId,
      scope,
      order: input.order ?? 0,
      component: input.component,
      props: input.props ?? {},
      routeKey,
    };

    updateRegistrations((current) => {
      const filtered = current.filter((entry) => entry.registrationId !== registrationId);
      filtered.push(nextEntry);
      return filtered;
    });

    return {
      ok: true,
      registrationId,
      replaced: Boolean(sameOwnerEntry),
    };
  }

  function unregister(registrationId: string): boolean {
    const previousLength = registrations.value.length;
    updateRegistrations((current) =>
      current.filter((entry) => entry.registrationId !== registrationId),
    );
    return registrations.value.length !== previousLength;
  }

  function unregisterEntry(ownerId: string, entryId: string): boolean {
    const previousLength = registrations.value.length;
    updateRegistrations((current) =>
      current.filter((entry) => !(entry.ownerId === ownerId && entry.entryId === entryId)),
    );
    return registrations.value.length !== previousLength;
  }

  function registerControlledComponent(
    input: NextHostControlledComponentRegistrationInput,
  ): NextHostSlotRegistrationResult {
    const definition = CONTROLLED_COMPONENT_REGISTRY[input.componentType];
    if (!definition) {
      return {
        ok: false,
        reason: "unknown-component",
      };
    }

    const slotId = input.slotId ?? definition.defaultSlotId;
    const allowedSlotIds = definition.allowedSlotIds ?? [definition.defaultSlotId];
    if (!allowedSlotIds.includes(slotId)) {
      return {
        ok: false,
        reason: "slot-not-allowed",
      };
    }

    return register({
      ownerId: input.ownerId,
      entryId: input.entryId,
      slotId,
      order: input.order ?? definition.defaultOrder ?? 0,
      component: definition.component,
      props: input.props,
    });
  }

  function unregisterOwner(ownerId: string): void {
    updateRegistrations((current) => current.filter((entry) => entry.ownerId !== ownerId));
  }

  function clearPageScopedRegistrations(routeKey: string): void {
    updateRegistrations((current) =>
      current.filter((entry) => !(entry.routeKey === routeKey && entry.scope === "page")),
    );
  }

  return {
    registrations,
    register,
    unregister,
    unregisterEntry,
    registerControlledComponent,
    unregisterOwner,
    clearPageScopedRegistrations,
  };
}

export function registerNextHostControlledComponent(
  type: string,
  definition: NextHostControlledComponentDefinition,
): void {
  CONTROLLED_COMPONENT_REGISTRY[type] = definition;
}

export function getRegisteredNextHostControlledComponentTypes(): string[] {
  return Object.keys(CONTROLLED_COMPONENT_REGISTRY).toSorted();
}

function getPageKind(route: ReturnType<typeof useRoute>): NextShellPageKind {
  return resolveNextPageKind(route.meta.pageKind);
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

  return Array.from(items.values()).toSorted((a, b) => a.label.localeCompare(b.label));
}

export function provideNextHostRuntime(): NextHostRuntimeContext {
  const route = useRoute();
  const sidebarItems = shallowRef<NextSidebarHostItem[]>([]);
  const homeCardDefinitions = shallowRef<NextHomeCardDefinitionRegistry>({});
  const homeCardRenderers = shallowRef<NextHomeCardRendererRegistry>({});
  const lifecycleEvent = shallowRef<{
    name: NextHostLifecycleEventName;
    payload: NextHostLifecyclePayload;
  } | null>(null);

  let routeSequence = 1;

  function buildPayload(): NextHostLifecyclePayload {
    return {
      route_key: `next:${routeSequence}`,
      page_kind: getPageKind(route),
      shell_id: "next",
      path: route.fullPath,
    };
  }

  let currentPayload: NextHostLifecyclePayload | null = buildPayload();
  const slots = createNextHostSlotManager(() => currentPayload?.route_key ?? null);

  async function hydrateSidebarSnapshot(): Promise<void> {
    if (!(await isPluginRuntimeUiBridgeAvailable())) {
      sidebarItems.value = [];
      return;
    }

    try {
      const snapshot = await getPluginSidebarSnapshot();
      sidebarItems.value = buildSidebarItems(snapshot);
    } catch {
      sidebarItems.value = [];
    }
  }

  function registerHomeCards(definitions: readonly NextHomeHostCardDefinition[]): void {
    const nextDefinitions = { ...homeCardDefinitions.value };
    const nextRenderers = { ...homeCardRenderers.value };

    for (const definition of definitions) {
      if (!isNextHomePluginCardKind(definition.kind)) {
        continue;
      }

      nextDefinitions[definition.kind] = {
        definition,
        renderer: definition.component,
      };
      nextRenderers[definition.kind] = definition.component;
    }

    homeCardDefinitions.value = nextDefinitions;
    homeCardRenderers.value = nextRenderers;
  }

  function registerHomeCardRenderers(registry: NextHomeCardRendererRegistry): void {
    const nextRegistry = { ...homeCardRenderers.value };

    for (const [rawKind, component] of Object.entries(registry)) {
      if (!component || !isNextHomePluginCardKind(rawKind)) {
        continue;
      }

      nextRegistry[rawKind] = component;
    }

    homeCardRenderers.value = nextRegistry;
  }

  onMounted(async () => {
    if (!currentPayload) {
      currentPayload = buildPayload();
    }

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
      slots.clearPageScopedRegistrations(currentPayload.route_key);
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
    slots.clearPageScopedRegistrations(currentPayload.route_key);
  });

  const context: NextHostRuntimeContext = {
    sidebarItems,
    slots,
    home: {
      cardDefinitions: homeCardDefinitions,
      cardRenderers: homeCardRenderers,
      registerCards: registerHomeCards,
      registerCardRenderers: registerHomeCardRenderers,
    },
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
