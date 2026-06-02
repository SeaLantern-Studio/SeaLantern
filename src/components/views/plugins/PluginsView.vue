<script setup lang="ts">
import SLDropzone from "@components/common/SLDropzone.vue";
import PluginChooserDialog from "@components/views/plugins/PluginChooserDialog.vue";
import PluginFeedbackDialogs from "@components/views/plugins/PluginFeedbackDialogs.vue";
import PluginList from "@components/views/plugins/PluginList.vue";
import PluginSettingsDialog from "@components/views/plugins/PluginSettingsDialog.vue";
import PluginsStatePanel from "@components/views/plugins/PluginsStatePanel.vue";
import PluginsToolbar from "@components/views/plugins/PluginsToolbar.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { usePluginsViewModel } from "@components/views/plugins/usePluginsViewModel";
import { i18n } from "@language";
import { Upload } from "lucide-vue-next";

const viewModel = usePluginsViewModel();
const pluginStore = viewModel.pluginStore;
const searchQuery = viewModel.searchQuery;
const {
  isDragging,
  chooserOpen,
  isInstalling,
  showDependencyModal,
  missingDependencies,
  installedPluginName,
  showBatchResultModal,
  batchInstallResult,
  installErrorMessage,
  openChooser,
  handleBatchInstall,
  pickFile,
  pickFolder,
} = viewModel.installer;
const safeMode = viewModel.installer.safeMode;
const checkingAllUpdates = viewModel.listActions.checkingAllUpdates;
const filteredPlugins = viewModel.page.filteredPlugins;
const {
  batchMode,
  selectedPlugins,
  showBatchDeleteDialog,
  showSingleDeleteDialog,
  singleDeletePluginName,
  toggleBatchMode,
  togglePluginSelection,
  selectAll,
  deselectAll,
  invertSelection,
  showBatchDeleteConfirm,
} = viewModel.selection;
const dependencies = viewModel.dependencies;
const { alertDialog, permissionWarning, closeAlertDialog } = viewModel.feedback;
const listActions = viewModel.listActions;
const {
  showSettingsModal,
  currentSettingsPlugin,
  settingsForm,
  savingSettings,
  openSettings,
  closeSettings,
} = viewModel.settingsDialog;
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
      :plugins="filteredPlugins"
      :batch-mode="batchMode"
      :selected-plugin-ids="selectedPlugins"
      :updates="pluginStore.updates"
      :icons="pluginStore.icons"
      :safe-mode="safeMode"
      :get-plugin-name="page.getPluginName"
      :get-plugin-description="page.getPluginDescription"
      :is-plugin-enabled="listActions.isPluginEnabled"
      :get-status-color="listActions.getStatusColor"
      :get-status-label="listActions.getStatusLabel"
      :has-settings="listActions.hasSettings"
      :has-missing-required-dependencies="dependencies.hasMissingRequiredDependencies"
      :has-missing-optional-dependencies="dependencies.hasMissingOptionalDependencies"
      :get-dependency-tooltip="dependencies.getDependencyTooltip"
      :get-plugin-menu-items="listActions.getPluginMenuItems"
      @select-all="selectAll(pluginStore.plugins)"
      @invert-selection="invertSelection(pluginStore.plugins)"
      @deselect-all="deselectAll"
      @batch-delete="showBatchDeleteConfirm"
      @toggle-plugin-selection="togglePluginSelection"
      @show-missing-dependencies="page.showMissingDependenciesModal"
      @menu-select="listActions.handleMenuSelect"
      @open-repository="page.openRepository"
      @open-settings="openSettings"
      @toggle-plugin="listActions.handleToggle"
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
      @confirm="listActions.confirmPermissionWarning"
      @cancel="listActions.cancelPermissionWarning"
    />

    <PluginFeedbackDialogs
      :show-single-delete-dialog="showSingleDeleteDialog"
      :single-delete-plugin-name="singleDeletePluginName"
      :show-batch-delete-dialog="showBatchDeleteDialog"
      :selected-count="selectedPlugins.size"
      :alert-dialog="alertDialog"
      :show-dependency-modal="showDependencyModal"
      :installed-plugin-name="installedPluginName"
      :missing-dependencies="missingDependencies"
      :show-batch-result-modal="showBatchResultModal"
      :batch-install-result="batchInstallResult"
      :get-dep-display-name="dependencies.getDepDisplayName"
      @close-single-delete="showSingleDeleteDialog = false"
      @confirm-single-delete="listActions.executeSingleDelete"
      @close-batch-delete="showBatchDeleteDialog = false"
      @confirm-batch-delete="listActions.executeBatchDelete"
      @close-alert="closeAlertDialog"
      @close-dependency="showDependencyModal = false"
      @go-market="page.handleGoToMarket"
      @close-batch-result="showBatchResultModal = false"
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
