<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import type { PendingPluginComponentCreate } from "@stores/plugin/pluginComponentBridge";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { ALLOWED_PLUGIN_COMPONENT_TYPES } from "@stores/plugin/pluginComponentBridgeShared";
import SLProgress from "@components/common/SLProgress.vue";
import {
  createPluginRuntimeHost,
  getPluginRuntimeSurface,
  getPluginUiContainer,
} from "@stores/plugin/pluginRuntimeDomShared";

const pluginStore = usePluginStore();

const componentMap: Record<string, any> = {
  SLProgress,
};

interface RenderedComponent {
  pluginId: string;
  id: string;
  type: string;
  props: Record<string, any>;
}

const renderedComponents = ref<RenderedComponent[]>([]);

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
    if (!componentMap[create.component_type]) {
      pluginLogger.warn("Component", "组件类型未映射到渲染器", {
        pluginId: create.plugin_id,
        componentType: create.component_type,
        componentId: create.component_id,
      });
      continue;
    }
    if (!renderedComponents.value.find((c) => c.id === create.component_id)) {
      renderedComponents.value.push({
        pluginId: create.plugin_id,
        id: create.component_id,
        type: create.component_type,
        props: create.props,
      });
    }
  }
}

function consumePendingDeletes(pluginId: string) {
  const deletes = safeConsumeDeletes(pluginId);
  for (const id of deletes) {
    const index = renderedComponents.value.findIndex((c) => c.id === id);
    if (index !== -1) {
      removeComponentHost(renderedComponents.value[index]);
      renderedComponents.value.splice(index, 1);
    }
  }
}

function processAllPendingComponents() {
  const plugins = (pluginStore.plugins as any)?.value ?? [];
  for (const plugin of plugins) {
    if (plugin.state === "enabled") {
      consumePendingCreates(plugin.manifest.id);
      consumePendingDeletes(plugin.manifest.id);
    }
  }
}

function getComponentProps(component: RenderedComponent) {
  const props: Record<string, any> = {};

  for (const [key, value] of Object.entries(component.props)) {
    props[key] = value;

    const kebabKey = key.replace(/([A-Z])/g, "-$1").toLowerCase();
    if (kebabKey !== key) {
      props[kebabKey] = value;
    }
  }

  if (!props["componentId"] && !props["component-id"]) {
    props["componentId"] = component.id;
  }

  return props;
}

function getComponentHostId(component: RenderedComponent): string {
  return `plugin-component-host-${component.pluginId}-${component.id}`;
}

function ensureComponentHost(component: RenderedComponent) {
  const existing = document.getElementById(getComponentHostId(component));
  if (existing) {
    return getPluginRuntimeSurface(existing);
  }

  const container = getPluginUiContainer();
  const host = createPluginRuntimeHost(component.pluginId, `component-${component.id}`);
  host.id = getComponentHostId(component);
  container.appendChild(host);
  return getPluginRuntimeSurface(host);
}

function removeComponentHost(component: RenderedComponent) {
  document.getElementById(getComponentHostId(component))?.remove();
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
  for (const component of renderedComponents.value) {
    removeComponentHost(component);
  }
});
</script>

<template>
  <div class="plugin-component-renderer" aria-hidden="true">
    <Teleport
      v-for="component in renderedComponents"
      :key="component.id"
      :to="ensureComponentHost(component) ?? 'body'"
    >
      <component :is="componentMap[component.type]" v-bind="getComponentProps(component)" />
    </Teleport>
  </div>
</template>

<style scoped>
.plugin-component-renderer {
  display: contents;
}
</style>
