import { computed } from "vue";
import type { PluginInfo } from "@type/plugin";
import type { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import type { usePluginListActions } from "@components/views/plugins/usePluginListActions";
import type { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import type { usePluginsViewPage } from "@components/views/plugins/usePluginsViewPage";

type PluginDependenciesModel = ReturnType<typeof usePluginDependencies>;
type PluginListActionsModel = ReturnType<typeof usePluginListActions>;
type PluginSelectionModel = ReturnType<typeof usePluginSelection>;
type PluginsViewPageModel = ReturnType<typeof usePluginsViewPage>;

interface UsePluginListBindingsOptions {
  plugins: () => PluginInfo[];
  updates: () => Record<string, any>;
  icons: () => Record<string, string>;
  safeMode: () => boolean;
  selection: PluginSelectionModel;
  dependencies: PluginDependenciesModel;
  listActions: PluginListActionsModel;
  page: PluginsViewPageModel;
  openSettings: (plugin: PluginInfo) => void;
}

export function usePluginListBindings(options: UsePluginListBindingsOptions) {
  const listProps = computed(() => ({
    plugins: options.page.filteredPlugins.value,
    batchMode: options.selection.batchMode.value,
    selectedPluginIds: options.selection.selectedPlugins.value,
    updates: options.updates(),
    icons: options.icons(),
    safeMode: options.safeMode(),
    getPluginName: options.page.getPluginName,
    getPluginDescription: options.page.getPluginDescription,
    isPluginEnabled: options.listActions.isPluginEnabled,
    getStatusColor: options.listActions.getStatusColor,
    getStatusLabel: options.listActions.getStatusLabel,
    hasSettings: options.listActions.hasSettings,
    hasMissingRequiredDependencies: options.dependencies.hasMissingRequiredDependencies,
    hasMissingOptionalDependencies: options.dependencies.hasMissingOptionalDependencies,
    getDependencyTooltip: options.dependencies.getDependencyTooltip,
    getPluginMenuItems: options.listActions.getPluginMenuItems,
  }));

  const listHandlers = {
    selectAll: () => options.selection.selectAll(options.plugins()),
    invertSelection: () => options.selection.invertSelection(options.plugins()),
    deselectAll: options.selection.deselectAll,
    batchDelete: options.selection.showBatchDeleteConfirm,
    togglePluginSelection: options.selection.togglePluginSelection,
    showMissingDependencies: options.page.showMissingDependenciesModal,
    menuSelect: options.listActions.handleMenuSelect,
    openRepository: options.page.openRepository,
    openSettings: options.openSettings,
    togglePlugin: options.listActions.handleToggle,
  };

  return {
    listProps,
    listHandlers,
  };
}
