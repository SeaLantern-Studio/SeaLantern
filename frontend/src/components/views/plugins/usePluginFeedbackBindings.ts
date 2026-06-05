import { computed } from "vue";
import type { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import type { usePluginFeedback } from "@components/views/plugins/usePluginFeedback";
import type { usePluginListActions } from "@components/views/plugins/usePluginListActions";
import type { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import type { usePluginsInstaller } from "@components/views/plugins/usePluginsInstaller";
import type { usePluginsViewPage } from "@components/views/plugins/usePluginsViewPage";

type PluginSelectionState = ReturnType<typeof usePluginSelection>;
type PluginFeedbackState = ReturnType<typeof usePluginFeedback>;
type PluginDependenciesState = ReturnType<typeof usePluginDependencies>;
type PluginListActionsState = ReturnType<typeof usePluginListActions>;
type PluginsInstallerState = ReturnType<typeof usePluginsInstaller>;
type PluginsViewPageState = ReturnType<typeof usePluginsViewPage>;

interface UsePluginFeedbackBindingsOptions {
  selection: PluginSelectionState;
  feedback: PluginFeedbackState;
  dependencies: PluginDependenciesState;
  listActions: PluginListActionsState;
  installer: PluginsInstallerState;
  page: PluginsViewPageState;
}

export function usePluginFeedbackBindings(options: UsePluginFeedbackBindingsOptions) {
  const feedbackDialogProps = computed(() => ({
    showSingleDeleteDialog: options.selection.showSingleDeleteDialog.value,
    singleDeletePluginName: options.selection.singleDeletePluginName.value,
    showBatchDeleteDialog: options.selection.showBatchDeleteDialog.value,
    selectedCount: options.selection.selectedPlugins.value.size,
    alertDialog: options.feedback.alertDialog.value,
    showDependencyModal: options.installer.showDependencyModal.value,
    installedPluginName: options.installer.installedPluginName.value,
    missingDependencies: options.installer.missingDependencies.value,
    showBatchResultModal: options.installer.showBatchResultModal.value,
    batchInstallResult: options.installer.batchInstallResult.value,
    getDepDisplayName: options.dependencies.getDepDisplayName,
  }));

  const feedbackDialogHandlers = {
    closeSingleDelete: () => {
      options.selection.showSingleDeleteDialog.value = false;
    },
    confirmSingleDelete: options.listActions.executeSingleDelete,
    closeBatchDelete: () => {
      options.selection.showBatchDeleteDialog.value = false;
    },
    confirmBatchDelete: options.listActions.executeBatchDelete,
    closeAlert: options.feedback.closeAlertDialog,
    closeDependency: () => {
      options.installer.showDependencyModal.value = false;
    },
    goMarket: options.page.handleGoToMarket,
    closeBatchResult: () => {
      options.installer.showBatchResultModal.value = false;
    },
  };

  return {
    feedbackDialogProps,
    feedbackDialogHandlers,
  };
}
