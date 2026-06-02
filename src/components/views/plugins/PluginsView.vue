<script setup lang="ts">
import { ref, computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import SLDropzone from "@components/common/SLDropzone.vue";
import PluginChooserDialog from "@components/views/plugins/PluginChooserDialog.vue";
import PluginFeedbackDialogs from "@components/views/plugins/PluginFeedbackDialogs.vue";
import PluginList from "@components/views/plugins/PluginList.vue";
import PluginSettingsDialog from "@components/views/plugins/PluginSettingsDialog.vue";
import PluginsStatePanel from "@components/views/plugins/PluginsStatePanel.vue";
import PluginsToolbar from "@components/views/plugins/PluginsToolbar.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import { usePluginFeedback } from "@components/views/plugins/usePluginFeedback";
import { usePluginListActions } from "@components/views/plugins/usePluginListActions";
import { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import { usePluginSettingsDialog } from "@components/views/plugins/usePluginSettingsDialog";
import { usePluginsInstaller } from "@components/views/plugins/usePluginsInstaller";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import type { PluginInfo } from "@type/plugin";
import { getLocalizedPluginName, getLocalizedPluginDescription } from "@type/plugin";
import { Upload } from "lucide-vue-next";

const pluginStore = usePluginStore();
const searchQuery = ref("");
const {
  isDragging,
  chooserOpen,
  safeMode,
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
} = usePluginsInstaller();
const {
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
} = usePluginSelection();
const {
  getDepDisplayName,
  hasMissingRequiredDependencies,
  getMissingRequiredDependencies,
  getMissingOptionalDependencies,
  hasMissingOptionalDependencies,
  getDependencyTooltip,
  getPluginDependencyViewModel,
} = usePluginDependencies(() => pluginStore.plugins);

const filteredPlugins = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return pluginStore.plugins;
  return pluginStore.plugins.filter((p) => {
    const id = p.manifest.id.toLowerCase();
    const name = getLocalizedPluginName(p.manifest, i18n.getLocale()).toLowerCase();
    const stateStr = (typeof p.state === "string" ? p.state : "error").toLowerCase();
    return id.includes(q) || name.includes(q) || stateStr.includes(q);
  });
});

const {
  alertDialog,
  permissionWarning,
  showAlert,
  closeAlertDialog,
  openPermissionWarning,
  closePermissionWarning,
  getErrorMessage,
} = usePluginFeedback();

const {
  checkingAllUpdates,
  handleToggle,
  confirmPermissionWarning,
  cancelPermissionWarning,
  executeSingleDelete,
  executeBatchDelete,
  handleCheckAllUpdates,
  getStatusColor,
  getStatusLabel,
  isPluginEnabled,
  hasSettings,
  getPluginMenuItems,
  handleMenuSelect,
  goToMarket,
} = usePluginListActions({
  plugins: () => pluginStore.plugins,
  clearError: () => {
    pluginStore.error = null;
  },
  togglePlugin: pluginStore.togglePlugin,
  checkUpdate: pluginStore.checkUpdate,
  checkAllUpdates: pluginStore.checkAllUpdates,
  deletePlugin: pluginStore.deletePlugin,
  deletePlugins: pluginStore.deletePlugins,
  showAlert,
  getErrorMessage,
  openPermissionWarning,
  closePermissionWarning,
  pendingPermissionPluginId: () => permissionWarning.value.pluginId,
  prepareSingleDelete,
  clearSingleDeleteState,
  pendingDeletePluginId: () => pendingDeletePluginId.value,
  selectedPluginIds: () => selectedPlugins.value,
  clearSelection: () => {
    selectedPlugins.value.clear();
  },
  closeSingleDeleteDialog: () => {
    showSingleDeleteDialog.value = false;
  },
  closeBatchDeleteDialog: () => {
    showBatchDeleteDialog.value = false;
  },
  exitBatchMode: () => {
    batchMode.value = false;
  },
});

const {
  showSettingsModal,
  currentSettingsPlugin,
  settingsForm,
  savingSettings,
  openSettings,
  closeSettings,
  saveSettings,
} = usePluginSettingsDialog({
  getPluginSettings: pluginStore.getPluginSettings,
  setPluginSettings: pluginStore.setPluginSettings,
  hasCapability: pluginStore.hasCapability,
  applyThemeProviderSettings: pluginStore.applyThemeProviderSettings,
  logError: (message, details) => {
    pluginLogger.error("PluginsView", message, details);
  },
});

function handleRefresh() {
  pluginStore.refreshPlugins();
}

function getPermissionLabel(perm: string): string {
  return i18n.t(`plugins.permission.${perm}`) !== `plugins.permission.${perm}`
    ? i18n.t(`plugins.permission.${perm}`)
    : perm;
}

function getPermissionDesc(perm: string): string {
  return i18n.t(`plugins.permission.${perm}_desc`) !== `plugins.permission.${perm}_desc`
    ? i18n.t(`plugins.permission.${perm}_desc`)
    : "";
}

function updateSettingsField(key: string, value: string | number | boolean) {
  settingsForm[key] = value;
}

function showMissingDependenciesModal(plugin: PluginInfo) {
  installedPluginName.value = plugin.manifest.name;
  const required = getMissingRequiredDependencies(plugin);
  const optional = getMissingOptionalDependencies(plugin);
  missingDependencies.value = [...required, ...optional];
  showDependencyModal.value = true;
}

function openRepository(url: string) {
  openUrl(url);
}

function getPluginName(plugin: PluginInfo): string {
  return getLocalizedPluginName(plugin.manifest, i18n.getLocale());
}

function getPluginDescription(plugin: PluginInfo): string {
  return getLocalizedPluginDescription(plugin.manifest, i18n.getLocale());
}

async function handleSaveSettings() {
  const errorMessage = await saveSettings();
  if (errorMessage) {
    showAlert(i18n.t("common.message_unknown_error"), errorMessage);
  }
}

function handleGoToMarket() {
  showDependencyModal.value = false;
  goToMarket();
}
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
      @check-all-updates="handleCheckAllUpdates"
      @refresh="handleRefresh"
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
      :get-plugin-name="getPluginName"
      :get-plugin-description="getPluginDescription"
      :is-plugin-enabled="isPluginEnabled"
      :get-status-color="getStatusColor"
      :get-status-label="getStatusLabel"
      :has-settings="hasSettings"
      :has-missing-required-dependencies="hasMissingRequiredDependencies"
      :has-missing-optional-dependencies="hasMissingOptionalDependencies"
      :get-dependency-tooltip="getDependencyTooltip"
      :get-plugin-menu-items="getPluginMenuItems"
      @select-all="selectAll(pluginStore.plugins)"
      @invert-selection="invertSelection(pluginStore.plugins)"
      @deselect-all="deselectAll"
      @batch-delete="showBatchDeleteConfirm"
      @toggle-plugin-selection="togglePluginSelection"
      @show-missing-dependencies="showMissingDependenciesModal"
      @menu-select="handleMenuSelect"
      @open-repository="openRepository"
      @open-settings="openSettings"
      @toggle-plugin="handleToggle"
    />

    <PluginSettingsDialog
      :visible="showSettingsModal"
      :plugin="currentSettingsPlugin"
      :field-values="settingsForm"
      :saving="savingSettings"
      :get-permission-label="getPermissionLabel"
      :get-permission-desc="getPermissionDesc"
      :dependency-view-model="
        currentSettingsPlugin ? getPluginDependencyViewModel(currentSettingsPlugin) : null
      "
      @close="closeSettings"
      @save="handleSaveSettings"
      @update-field="updateSettingsField"
    />

    <SLPermissionDialog
      :show="permissionWarning.show"
      :plugin-name="permissionWarning.pluginName"
      :permissions="permissionWarning.permissions"
      @confirm="confirmPermissionWarning"
      @cancel="cancelPermissionWarning"
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
      :get-dep-display-name="getDepDisplayName"
      @close-single-delete="showSingleDeleteDialog = false"
      @confirm-single-delete="executeSingleDelete"
      @close-batch-delete="showBatchDeleteDialog = false"
      @confirm-batch-delete="executeBatchDelete"
      @close-alert="closeAlertDialog"
      @close-dependency="showDependencyModal = false"
      @go-market="handleGoToMarket"
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
