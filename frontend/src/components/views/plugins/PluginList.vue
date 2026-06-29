<script setup lang="ts">
import PluginListBatchBar from "./PluginListBatchBar.vue";
import PluginListCard from "./PluginListCard.vue";
import type { PluginInfo, PluginState, PluginUpdateInfo } from "@type/plugin";
import type { PluginMenuItem } from "./pluginListShared";

defineProps<{
  plugins: PluginInfo[];
  batchMode: boolean;
  selectedPluginIds: Set<string>;
  updates: Record<string, PluginUpdateInfo>;
  icons: Record<string, string>;
  safeMode: boolean;
  getPluginName: (plugin: PluginInfo) => string;
  getPluginDescription: (plugin: PluginInfo) => string;
  isPluginEnabled: (state: PluginState) => boolean;
  getStatusColor: (state: PluginState) => string;
  getStatusLabel: (state: PluginState) => string;
  hasSettings: (plugin: PluginInfo) => boolean;
  canTogglePlugin: (plugin: PluginInfo) => boolean;
  hasMissingRequiredDependencies: (plugin: PluginInfo) => boolean;
  hasMissingOptionalDependencies: (plugin: PluginInfo) => boolean;
  getDependencyTooltip: (plugin: PluginInfo) => string;
  getPluginMenuItems: (pluginId: string) => PluginMenuItem[];
}>();

const emit = defineEmits<{
  (e: "select-all"): void;
  (e: "invert-selection"): void;
  (e: "deselect-all"): void;
  (e: "batch-delete"): void;
  (e: "toggle-plugin-selection", pluginId: string): void;
  (e: "show-missing-dependencies", plugin: PluginInfo): void;
  (e: "menu-select", item: { id: string | number }, pluginId: string): void;
  (e: "open-repository", url: string): void;
  (e: "open-settings", plugin: PluginInfo): void;
  (e: "toggle-plugin", pluginId: string, nextEnabled: boolean): void;
}>();

function handleMenuSelect(item: { id: string | number }, pluginId: string) {
  emit("menu-select", item, pluginId);
}

function handleTogglePlugin(pluginId: string, nextEnabled: boolean) {
  emit("toggle-plugin", pluginId, nextEnabled);
}
</script>

<template>
  <div>
    <PluginListBatchBar
      v-if="batchMode"
      :selected-count="selectedPluginIds.size"
      @select-all="emit('select-all')"
      @invert-selection="emit('invert-selection')"
      @deselect-all="emit('deselect-all')"
      @batch-delete="emit('batch-delete')"
    />

    <div class="plugin-grid">
      <PluginListCard
        v-for="plugin in plugins"
        :key="plugin.manifest.id"
        :plugin="plugin"
        :batch-mode="batchMode"
        :selected="selectedPluginIds.has(plugin.manifest.id)"
        :update-info="updates[plugin.manifest.id]"
        :icon-url="icons[plugin.manifest.id]"
        :safe-mode="safeMode"
        :get-plugin-name="getPluginName"
        :get-plugin-description="getPluginDescription"
        :is-plugin-enabled="isPluginEnabled"
        :get-status-color="getStatusColor"
        :get-status-label="getStatusLabel"
        :has-settings="hasSettings"
        :can-toggle-plugin="canTogglePlugin"
        :has-missing-required-dependencies="hasMissingRequiredDependencies"
        :has-missing-optional-dependencies="hasMissingOptionalDependencies"
        :get-dependency-tooltip="getDependencyTooltip"
        :menu-items="getPluginMenuItems(plugin.manifest.id)"
        @toggle-selection="emit('toggle-plugin-selection', $event)"
        @show-missing-dependencies="emit('show-missing-dependencies', $event)"
        @menu-select="handleMenuSelect"
        @open-repository="emit('open-repository', $event)"
        @open-settings="emit('open-settings', $event)"
        @toggle-plugin="handleTogglePlugin"
      />
    </div>
  </div>
</template>

<style scoped>
.plugin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--sl-space-md);
}
</style>
