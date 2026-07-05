<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import type { PendingPluginComponentCreate } from "@stores/plugin/pluginComponentBridge";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  ALLOWED_PLUGIN_COMPONENT_TYPES,
  getPluginComponentHostOwnerId,
} from "@stores/plugin/pluginComponentBridgeShared";
import SLProgress from "@components/common/SLProgress.vue";
import { NEXT_HOST_SLOT_IDS, type NextHostSlotId } from "@src/contracts/slots";
import { registerNextHostControlledComponent, useNextHostRuntime } from "@src/host/runtime";

const pluginStore = usePluginStore();
const nextHostRuntime = useNextHostRuntime();

registerNextHostControlledComponent("SLProgress", {
  component: SLProgress,
  defaultSlotId: NEXT_HOST_SLOT_IDS.pageContentAfter,
  allowedSlotIds: [
    NEXT_HOST_SLOT_IDS.pageHeaderActions,
    NEXT_HOST_SLOT_IDS.pageContentBefore,
    NEXT_HOST_SLOT_IDS.pageContentAfter,
  ],
  defaultOrder: 300,
});

function safeConsumeCreates(pluginId: string): PendingPluginComponentCreate[] {
  const fn = (pluginStore as any).consumePendingComponentCreates;
  if (typeof fn !== "function") return [];
  return fn(pluginId) as PendingPluginComponentCreate[];
}

function safeConsumeDeletes(pluginId: string): string[] {
  const fn = (pluginStore as any).consumePendingComponentDeletes;
  if (typeof fn !== "function") return [];
  return fn(pluginId) as string[];
}

function consumePendingCreates(pluginId: string) {
  const creates = safeConsumeCreates(pluginId);
  for (const create of creates) {
    if (!ALLOWED_PLUGIN_COMPONENT_TYPES.has(create.component_type)) {
      pluginLogger.warn("Component", "已跳过未开放组件渲染", {
        pluginId: create.plugin_id,
        componentType: create.component_type,
        componentId: create.component_id,
      });
      continue;
    }

    const registration = nextHostRuntime.slots.registerControlledComponent({
      ownerId: getPluginComponentHostOwnerId(create.plugin_id),
      entryId: create.component_id,
      componentType: create.component_type,
      slotId: resolveComponentSlotId(create),
      order: resolveComponentOrder(create),
      props: buildComponentProps(create),
    });

    if (!registration.ok) {
      pluginLogger.warn("Component", "受控组件注册已拒绝", {
        pluginId: create.plugin_id,
        componentType: create.component_type,
        componentId: create.component_id,
        reason: registration.reason,
        conflictWith: registration.conflictWith,
      });
    }
  }
}

function consumePendingDeletes(pluginId: string) {
  const deletes = safeConsumeDeletes(pluginId);
  for (const id of deletes) {
    nextHostRuntime.slots.unregisterEntry(getPluginComponentHostOwnerId(pluginId), id);
  }
}

function processAllPendingComponents() {
  const plugins = (pluginStore.plugins as any)?.value ?? [];
  for (const plugin of plugins) {
    if (plugin.state === "enabled") {
      consumePendingCreates(plugin.manifest.id);
    }

    consumePendingDeletes(plugin.manifest.id);
  }
}

function buildComponentProps(component: PendingPluginComponentCreate) {
  const props: Record<string, any> = {};

  for (const [key, value] of Object.entries(component.props)) {
    if (key === "slotId" || key === "order") {
      continue;
    }

    props[key] = value;

    const kebabKey = key.replace(/([A-Z])/g, "-$1").toLowerCase();
    if (kebabKey !== key) {
      props[kebabKey] = value;
    }
  }

  if (!props["componentId"] && !props["component-id"]) {
    props["componentId"] = component.component_id;
  }

  return props;
}

function resolveComponentSlotId(
  component: PendingPluginComponentCreate,
): NextHostSlotId | undefined {
  const raw = component.props.slotId;
  if (
    raw === NEXT_HOST_SLOT_IDS.pageHeaderActions ||
    raw === NEXT_HOST_SLOT_IDS.pageContentBefore ||
    raw === NEXT_HOST_SLOT_IDS.pageContentAfter
  ) {
    return raw;
  }

  return undefined;
}

function resolveComponentOrder(component: PendingPluginComponentCreate): number | undefined {
  const raw = component.props.order;
  if (typeof raw === "number" && Number.isFinite(raw)) {
    return raw;
  }

  return undefined;
}

watch(
  () => pluginStore.pendingComponentVersion,
  () => {
    processAllPendingComponents();
  },
);

onMounted(() => {
  processAllPendingComponents();
});

onUnmounted(() => {
  const plugins = (pluginStore.plugins as any)?.value ?? [];
  for (const plugin of plugins) {
    nextHostRuntime.slots.unregisterOwner(getPluginComponentHostOwnerId(plugin.manifest.id));
  }
});
</script>

<template>
  <div class="plugin-component-renderer" aria-hidden="true" />
</template>

<style scoped>
.plugin-component-renderer {
  display: contents;
}
</style>
