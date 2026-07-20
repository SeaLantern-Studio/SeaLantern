<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { usePluginStore } from "@stores/pluginStore";

type PendingCreate = {
  component_type: string;
  component_id: string;
  props: Record<string, any>;
};

const pluginStore = usePluginStore();

// 动态组件映射 → CmzYa 全局注册名称 (kebab-case)
const componentMap: Record<string, string> = {
  SLCard: "cmz-card",
  SLButton: "cmz-button",
  SLInput: "cmz-input",
  SLCheckbox: "cmz-checkbox",
  SLSwitch: "cmz-switch",
  SLProgress: "cmz-progress",
  SLSelect: "cmz-select",
  SLTabs: "cmz-tab-bar",
  SLTabBar: "cmz-tab-bar",
};

interface RenderedComponent {
  id: string;
  type: string;
  props: Record<string, any>;
}

const renderedComponents = ref<RenderedComponent[]>([]);

function safeConsumeCreates(pluginId: string): PendingCreate[] {
  const fn = (pluginStore as any).consumePendingComponentCreates;
  if (typeof fn !== "function") return [];
  return fn(pluginId) as PendingCreate[];
}

function safeConsumeDeletes(pluginId: string): string[] {
  const fn = (pluginStore as any).consumePendingComponentDeletes;
  if (typeof fn !== "function") return [];
  return fn(pluginId) as string[];
}

function consumePendingCreates(pluginId: string) {
  const creates = safeConsumeCreates(pluginId);
  for (const create of creates) {
    if (!renderedComponents.value.find((c) => c.id === create.component_id)) {
      renderedComponents.value.push({
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

let intervalId: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  processAllPendingComponents();
  intervalId = setInterval(() => {
    processAllPendingComponents();
  }, 300);
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
});
</script>

<template>
  <div class="plugin-component-renderer">
    <component
      v-for="component in renderedComponents"
      :key="component.id"
      :is="componentMap[component.type] || component.type"
      v-bind="getComponentProps(component)"
    />
  </div>
</template>

<style scoped>
.plugin-component-renderer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 9999;
}

.plugin-component-renderer > * {
  pointer-events: auto;
}
</style>
