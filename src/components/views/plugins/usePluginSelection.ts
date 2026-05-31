import { ref } from "vue";
import type { PluginInfo } from "@type/plugin";

export function usePluginSelection() {
  const batchMode = ref(false);
  const selectedPlugins = ref<Set<string>>(new Set());
  const showBatchDeleteDialog = ref(false);
  const pendingDeletePluginId = ref<string | null>(null);
  const showSingleDeleteDialog = ref(false);
  const singleDeletePluginName = ref("");

  function toggleBatchMode() {
    batchMode.value = !batchMode.value;
    if (!batchMode.value) {
      selectedPlugins.value.clear();
      selectedPlugins.value = new Set(selectedPlugins.value);
    }
  }

  function togglePluginSelection(pluginId: string) {
    if (selectedPlugins.value.has(pluginId)) {
      selectedPlugins.value.delete(pluginId);
    } else {
      selectedPlugins.value.add(pluginId);
    }
    selectedPlugins.value = new Set(selectedPlugins.value);
  }

  function selectAll(plugins: Readonly<PluginInfo[]>) {
    selectedPlugins.value = new Set(plugins.map((plugin) => plugin.manifest.id));
  }

  function deselectAll() {
    selectedPlugins.value.clear();
    selectedPlugins.value = new Set(selectedPlugins.value);
  }

  function invertSelection(plugins: Readonly<PluginInfo[]>) {
    const nextSelection = new Set<string>();
    for (const plugin of plugins) {
      if (!selectedPlugins.value.has(plugin.manifest.id)) {
        nextSelection.add(plugin.manifest.id);
      }
    }
    selectedPlugins.value = nextSelection;
  }

  function showBatchDeleteConfirm() {
    showBatchDeleteDialog.value = true;
  }

  function prepareSingleDelete(plugin: PluginInfo | undefined, pluginId: string) {
    pendingDeletePluginId.value = pluginId;
    singleDeletePluginName.value = plugin?.manifest?.name || pluginId;
    showSingleDeleteDialog.value = true;
  }

  function clearSingleDeleteState() {
    pendingDeletePluginId.value = null;
    showSingleDeleteDialog.value = false;
    singleDeletePluginName.value = "";
  }

  return {
    batchMode,
    selectedPlugins,
    showBatchDeleteDialog,
    pendingDeletePluginId,
    showSingleDeleteDialog,
    singleDeletePluginName,
    toggleBatchMode,
    togglePluginSelection,
    selectAll,
    deselectAll,
    invertSelection,
    showBatchDeleteConfirm,
    prepareSingleDelete,
    clearSingleDeleteState,
  };
}
