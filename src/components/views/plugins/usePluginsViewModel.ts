import { ref } from "vue";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import { usePluginFeedback } from "@components/views/plugins/usePluginFeedback";
import { usePluginFeedbackBindings } from "@components/views/plugins/usePluginFeedbackBindings";
import { usePluginListActions } from "@components/views/plugins/usePluginListActions";
import { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import { usePluginSettingsDialog } from "@components/views/plugins/usePluginSettingsDialog";
import { usePluginsViewPage } from "@components/views/plugins/usePluginsViewPage";
import { usePluginsInstaller } from "@components/views/plugins/usePluginsInstaller";

export function usePluginsViewModel() {
  const pluginStore = usePluginStore();
  const searchQuery = ref("");

  const installer = usePluginsInstaller();
  const selection = usePluginSelection();
  const dependencies = usePluginDependencies(() => pluginStore.plugins);
  const feedback = usePluginFeedback();

  const listActions = usePluginListActions({
    plugins: () => pluginStore.plugins,
    clearError: () => {
      pluginStore.error = null;
    },
    togglePlugin: pluginStore.togglePlugin,
    checkUpdate: pluginStore.checkUpdate,
    checkAllUpdates: pluginStore.checkAllUpdates,
    deletePlugin: pluginStore.deletePlugin,
    deletePlugins: pluginStore.deletePlugins,
    showAlert: feedback.showAlert,
    getErrorMessage: feedback.getErrorMessage,
    openPermissionWarning: feedback.openPermissionWarning,
    closePermissionWarning: feedback.closePermissionWarning,
    pendingPermissionPluginId: () => feedback.permissionWarning.value.pluginId,
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

  const settingsDialog = usePluginSettingsDialog({
    getPluginSettings: pluginStore.getPluginSettings,
    setPluginSettings: pluginStore.setPluginSettings,
    logError: (message, details) => {
      pluginLogger.error("PluginsView", message, details);
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

  return {
    pluginStore,
    searchQuery,
    installer,
    selection,
    dependencies,
    feedback,
    feedbackBindings,
    listActions,
    settingsDialog,
    page,
  };
}
