import { usePluginFeedbackBindings } from "@components/views/plugins/usePluginFeedbackBindings";
import { usePluginListBindings } from "@components/views/plugins/usePluginListBindings";
import { usePluginListActions } from "@components/views/plugins/usePluginListActions";
import { usePluginsViewPage } from "@components/views/plugins/usePluginsViewPage";
import type { usePluginsViewState } from "@components/views/plugins/usePluginsViewState";

type PluginsViewState = ReturnType<typeof usePluginsViewState>;

interface UsePluginsViewBindingsOptions {
  state: PluginsViewState;
}

export function usePluginsViewBindings(options: UsePluginsViewBindingsOptions) {
  const { pluginStore, searchQuery, installer, selection, dependencies, feedback, settingsDialog } =
    options.state;

  const listActions = usePluginListActions({
    plugins: () => pluginStore.plugins,
    clearError: () => {
      pluginStore.error = null;
    },
    togglePlugin: pluginStore.togglePlugin,
    confirmEnablePlugin: pluginStore.confirmEnablePlugin,
    checkUpdate: pluginStore.checkUpdate,
    checkAllUpdates: pluginStore.checkAllUpdates,
    deletePlugin: pluginStore.deletePlugin,
    deletePlugins: pluginStore.deletePlugins,
    showAlert: feedback.showAlert,
    getErrorMessage: feedback.getErrorMessage,
    openPermissionWarning: feedback.openPermissionWarning,
    closePermissionWarning: feedback.closePermissionWarning,
    pendingPermissionPluginId: () => feedback.permissionWarning.value.pluginId,
    pendingPermissionGrantScope: () => feedback.permissionWarning.value.grantScope,
    prepareSingleDelete: selection.prepareSingleDelete,
    clearSingleDeleteState: selection.clearSingleDeleteState,
    pendingDeletePluginId: () => selection.pendingDeletePluginId.value,
    selectedPluginIds: () => selection.selectedPlugins.value,
    clearSelection: () => {
      selection.selectedPlugins.value.clear();
    },
    closeSingleDeleteDialog: () => {
      selection.showSingleDeleteDialog.value = false;
    },
    closeBatchDeleteDialog: () => {
      selection.showBatchDeleteDialog.value = false;
    },
    exitBatchMode: () => {
      selection.batchMode.value = false;
    },
  });

  const page = usePluginsViewPage({
    plugins: () => pluginStore.plugins,
    searchQuery: () => searchQuery.value,
    refreshPlugins: pluginStore.refreshPlugins,
    getMissingRequiredDependencies: dependencies.getMissingRequiredDependencies,
    getMissingOptionalDependencies: dependencies.getMissingOptionalDependencies,
    saveSettings: settingsDialog.saveSettings,
    showAlert: (title, message) => {
      feedback.showAlert(title, message ?? "");
    },
    goToMarket: listActions.goToMarket,
    settingsForm: settingsDialog.settingsForm,
    setInstalledPluginName: (name) => {
      installer.installedPluginName.value = name;
    },
    setMissingDependencies: (dependenciesList) => {
      installer.missingDependencies.value = dependenciesList;
    },
    setShowDependencyModal: (show) => {
      installer.showDependencyModal.value = show;
    },
  });

  const feedbackBindings = usePluginFeedbackBindings({
    selection,
    feedback,
    dependencies,
    listActions,
    installer,
    page,
  });

  const listBindings = usePluginListBindings({
    plugins: () => pluginStore.plugins,
    updates: () => pluginStore.updates,
    icons: () => pluginStore.icons,
    safeMode: () => installer.safeMode.value,
    selection,
    dependencies,
    listActions,
    page,
    openSettings: settingsDialog.openSettings,
  });

  return {
    listActions,
    page,
    feedbackBindings,
    listBindings,
  };
}
