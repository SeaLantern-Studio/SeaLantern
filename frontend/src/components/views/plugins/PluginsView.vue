<script setup lang="ts">
import SLDropzone from "@components/common/SLDropzone.vue";
import PluginChooserDialog from "@components/plugin/installer/PluginChooserDialog.vue";
import PluginFeedbackDialogs from "@components/views/plugins/PluginFeedbackDialogs.vue";
import PluginList from "@components/views/plugins/PluginList.vue";
import PluginSettingsDialog from "@components/views/plugins/PluginSettingsDialog.vue";
import PluginsStatePanel from "@components/views/plugins/PluginsStatePanel.vue";
import PluginsToolbar from "@components/views/plugins/PluginsToolbar.vue";
import UiShellPanel from "@components/views/plugins/UiShellPanel.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { usePluginsViewModel } from "@components/views/plugins/usePluginsViewModel";
import { i18n } from "@language";
import { Upload } from "@lucide/vue";

const viewModel = usePluginsViewModel();
const pluginStore = viewModel.pluginStore;
const searchQuery = viewModel.searchQuery;
const listBindings = viewModel.listBindings;
const {
  isDragging,
  chooserOpen,
  isInstalling,
  installErrorMessage,
  openChooser,
  handleBatchInstall,
  pickFile,
  pickFolder,
} = viewModel.installer;
const checkingAllUpdates = viewModel.listActions.checkingAllUpdates;
const { batchMode, toggleBatchMode } = viewModel.selection;
const dependencies = viewModel.dependencies;
const { permissionWarning } = viewModel.feedback;
const feedbackBindings = viewModel.feedbackBindings;
const listActions = viewModel.listActions;
const { showSettingsModal, currentSettingsPlugin, settingsForm, savingSettings, closeSettings } =
  viewModel.settingsDialog;
const page = viewModel.page;
</script>

<template>
  <div class="plugins-view">
    <PluginsToolbar
      :search-query="searchQuery"
      :batch-mode="batchMode"
      :checking-all-updates="checkingAllUpdates"
      :loading="pluginStore.loading"
      @update:search-query="searchQuery = $event"
      @toggle-batch-mode="toggleBatchMode"
      @check-all-updates="listActions.handleCheckAllUpdates"
      @refresh="page.handleRefresh"
    />

    <UiShellPanel />

    <SLDropzone
      class="plugins-dropzone"
      :is-dragging="isDragging"
      :loading="isInstalling"
      :placeholder="i18n.t('plugins.drag_hint')"
      accept-folders
      accept-files
      :file-extensions="['.zip', '.json']"
      multiple
      @click="openChooser"
      @drop-multiple="handleBatchInstall"
    >
      <template #icon>
        <Upload :size="24" :stroke-width="1.5" />
      </template>
    </SLDropzone>

    <PluginChooserDialog
      :open="chooserOpen"
      @update:open="chooserOpen = $event"
      @pick-file="pickFile"
      @pick-folder="pickFolder"
    />

    <PluginsStatePanel
      :error-message="pluginStore.error || installErrorMessage"
      :loading="pluginStore.loading"
      :has-plugins="pluginStore.plugins.length > 0"
    />

    <PluginList
      v-if="pluginStore.plugins.length > 0"
      v-bind="listBindings.listProps.value"
      @select-all="listBindings.listHandlers.selectAll"
      @invert-selection="listBindings.listHandlers.invertSelection"
      @deselect-all="listBindings.listHandlers.deselectAll"
      @batch-delete="listBindings.listHandlers.batchDelete"
      @toggle-plugin-selection="listBindings.listHandlers.togglePluginSelection"
      @show-missing-dependencies="listBindings.listHandlers.showMissingDependencies"
      @menu-select="listBindings.listHandlers.menuSelect"
      @open-repository="listBindings.listHandlers.openRepository"
      @open-settings="listBindings.listHandlers.openSettings"
      @toggle-plugin="listBindings.listHandlers.togglePlugin"
    />

    <PluginSettingsDialog
      :visible="showSettingsModal"
      :plugin="currentSettingsPlugin"
      :field-values="settingsForm"
      :saving="savingSettings"
      :get-permission-label="page.getPermissionLabel"
      :get-permission-desc="page.getPermissionDesc"
      :dependency-view-model="
        currentSettingsPlugin
          ? dependencies.getPluginDependencyViewModel(currentSettingsPlugin)
          : null
      "
      @close="closeSettings"
      @save="page.handleSaveSettings"
      @update-field="page.updateSettingsField"
    />

    <SLPermissionDialog
      :show="permissionWarning.show"
      :plugin-name="permissionWarning.pluginName"
      :permissions="permissionWarning.permissions"
      :title="permissionWarning.title"
      :message="permissionWarning.message"
      :confirm-text="permissionWarning.confirmText"
      :confirm-variant="permissionWarning.confirmVariant"
      @confirm="listActions.confirmPermissionWarning"
      @cancel="listActions.cancelPermissionWarning"
    />

    <PluginFeedbackDialogs
      v-bind="feedbackBindings.feedbackDialogProps.value"
      @close-single-delete="feedbackBindings.feedbackDialogHandlers.closeSingleDelete"
      @confirm-single-delete="feedbackBindings.feedbackDialogHandlers.confirmSingleDelete"
      @close-batch-delete="feedbackBindings.feedbackDialogHandlers.closeBatchDelete"
      @confirm-batch-delete="feedbackBindings.feedbackDialogHandlers.confirmBatchDelete"
      @close-alert="feedbackBindings.feedbackDialogHandlers.closeAlert"
      @close-dependency="feedbackBindings.feedbackDialogHandlers.closeDependency"
      @go-market="feedbackBindings.feedbackDialogHandlers.goMarket"
      @close-batch-result="feedbackBindings.feedbackDialogHandlers.closeBatchResult"
    />
  </div>
</template>

<style scoped>
.plugins-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  min-height: 100%;
  flex: 1;
}

.plugins-dropzone {
  margin-bottom: var(--sl-space-md);
}

.plugins-dropzone :deep(.sl-dropzone) {
  justify-content: center;
  flex-direction: column;
  padding: var(--sl-space-lg);
}

.plugins-dropzone :deep(.sl-dropzone-content) {
  align-items: center;
  text-align: center;
}

.plugins-dropzone :deep(.sl-dropzone-title) {
  text-align: center;
}
</style>
