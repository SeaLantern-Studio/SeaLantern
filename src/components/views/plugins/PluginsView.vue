<script setup lang="ts">
import { ref, computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import SLMenu from "@components/common/SLMenu.vue";
import SLDropzone from "@components/common/SLDropzone.vue";
import PluginChooserDialog from "@components/views/plugins/PluginChooserDialog.vue";
import PluginFeedbackDialogs from "@components/views/plugins/PluginFeedbackDialogs.vue";
import PluginPermissionPanel from "@components/plugin/PluginPermissionPanel.vue";
import PluginSettingsDialog from "@components/views/plugins/PluginSettingsDialog.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { usePluginDependencies } from "@components/views/plugins/usePluginDependencies";
import { usePluginFeedback } from "@components/views/plugins/usePluginFeedback";
import { usePluginSelection } from "@components/views/plugins/usePluginSelection";
import { usePluginSettingsDialog } from "@components/views/plugins/usePluginSettingsDialog";
import { usePluginsInstaller } from "@components/views/plugins/usePluginsInstaller";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import type { PluginState, PluginInfo } from "@type/plugin";
import {
  hasDangerousPermissions,
  getLocalizedPluginName,
  getLocalizedPluginDescription,
} from "@type/plugin";
import {
  Upload,
  Layers,
  ShieldAlert,
  MoreVertical,
  Github,
  Settings,
  Trash2,
  RefreshCw,
} from "lucide-vue-next";

const router = useRouter();
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

const checkingUpdate = ref<string | null>(null);
const checkingAllUpdates = ref(false);

function handleRefresh() {
  pluginStore.refreshPlugins();
}

async function handleToggle(id: string, currentEnabled: boolean) {
  pluginStore.error = null;

  if (!currentEnabled) {
    const plugin = pluginStore.plugins.find((p) => p.manifest.id === id);
    const permissions = plugin?.manifest.permissions || [];
    if (hasDangerousPermissions(permissions)) {
      openPermissionWarning(id, plugin?.manifest.name || id, permissions);
      return;
    }
  }

  await doTogglePlugin(id, !currentEnabled);
}

async function confirmPermissionWarning() {
  const { pluginId } = permissionWarning.value;
  closePermissionWarning();
  await doTogglePlugin(pluginId, true);
}

function cancelPermissionWarning() {
  closePermissionWarning();
}

async function doTogglePlugin(id: string, enable: boolean) {
  const result = await pluginStore.togglePlugin(id, enable);

  if (!result.success && result.error) {
    showAlert(i18n.t("plugins.enable_failed"), result.error);
  } else if (result.disabledPlugins && result.disabledPlugins.length > 0) {
    const plugin = pluginStore.plugins.find((p) => p.manifest.id === id);
    const pluginName = plugin?.manifest.name || id;
    const disabledNames = result.disabledPlugins.map((depId) => {
      const dep = pluginStore.plugins.find((p) => p.manifest.id === depId);
      return dep?.manifest.name || depId;
    });
    showAlert(
      i18n.t("plugins.plugin_disabled"),
      i18n.t("plugins.plugin_disabled_desc", { name: pluginName, deps: disabledNames.join(", ") }),
    );
  }
}

function getStatusColor(state: PluginState): string {
  if (typeof state === "object" && "error" in state) {
    return "var(--sl-error)";
  }
  switch (state) {
    case "enabled":
      return "var(--sl-success)";
    case "disabled":
      return "var(--sl-text-tertiary)";
    case "loaded":
      return "var(--sl-info)";
    default:
      return "var(--sl-text-secondary)";
  }
}

function getStatusLabel(state: PluginState): string {
  if (typeof state === "object" && "error" in state) {
    return i18n.t("plugins.status.error");
  }
  switch (state) {
    case "enabled":
      return i18n.t("plugins.status.enabled");
    case "disabled":
      return i18n.t("plugins.status.disabled");
    case "loaded":
      return i18n.t("plugins.status.loaded");
    default:
      return String(state);
  }
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

function isPluginEnabled(state: PluginState): boolean {
  return state === "enabled";
}

function hasSettings(plugin: PluginInfo): boolean {
  return !!(plugin.manifest.settings && plugin.manifest.settings.length > 0);
}

function showMissingDependenciesModal(plugin: PluginInfo) {
  installedPluginName.value = plugin.manifest.name;
  const required = getMissingRequiredDependencies(plugin);
  const optional = getMissingOptionalDependencies(plugin);
  missingDependencies.value = [...required, ...optional];
  showDependencyModal.value = true;
}

function getPluginMenuItems(pluginId: string) {
  return [
    {
      id: "check_update",
      label: i18n.t("plugins.menu.check_update"),
      icon: RefreshCw,
      disabled: checkingUpdate.value === pluginId,
    },
    { id: "divider", label: "", divider: true },
    {
      id: "delete",
      label: i18n.t("plugins.menu.delete"),
      icon: Trash2,
      danger: true,
    },
  ];
}

async function handleMenuSelect(item: { id: string | number }, pluginId: string) {
  switch (item.id) {
    case "check_update":
      await handleCheckUpdate(pluginId);
      break;
    case "delete":
      await handleDelete(pluginId);
      break;
  }
}

async function handleCheckUpdate(pluginId: string) {
  checkingUpdate.value = pluginId;
  try {
    const update = await pluginStore.checkUpdate(pluginId);
    if (update) {
      showAlert(
        i18n.t("plugins.new_version_found"),
        `${i18n.t("plugins.latest_version")}: ${update.latest_version}\n${i18n.t("plugins.current_version")}: ${update.current_version}`,
      );
    } else {
      showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.already_latest"));
    }
  } finally {
    checkingUpdate.value = null;
  }
}

async function handleDelete(pluginId: string) {
  const plugin = pluginStore.plugins.find((p) => p.manifest.id === pluginId);
  if (plugin?.state === "enabled") {
    showAlert(i18n.t("plugins.cannot_delete_enabled"), plugin.manifest.name);
    return;
  }

  prepareSingleDelete(plugin, pluginId);
}

async function executeSingleDelete(deleteData: boolean) {
  showSingleDeleteDialog.value = false;
  if (!pendingDeletePluginId.value) return;
  try {
    await pluginStore.deletePlugin(pendingDeletePluginId.value, deleteData);
  } catch (e) {
    showAlert(i18n.t("common.message_unknown_error"), getErrorMessage(e));
  } finally {
    clearSingleDeleteState();
  }
}

async function executeBatchDelete(deleteData: boolean) {
  showBatchDeleteDialog.value = false;
  const ids = Array.from(selectedPlugins.value);

  const enabledNames = ids
    .map((id) => pluginStore.plugins.find((p) => p.manifest.id === id))
    .filter((p) => p?.state === "enabled")
    .map((p) => p!.manifest.name);
  if (enabledNames.length > 0) {
    showAlert(i18n.t("plugins.cannot_delete_enabled"), enabledNames.join(", "));
    return;
  }

  try {
    await pluginStore.deletePlugins(ids, deleteData);
    selectedPlugins.value.clear();
    batchMode.value = false;
  } catch (e) {
    showAlert(i18n.t("common.message_unknown_error"), getErrorMessage(e));
  }
}

async function handleCheckAllUpdates() {
  checkingAllUpdates.value = true;
  try {
    const updates = await pluginStore.checkAllUpdates();
    if (updates.length > 0) {
      showAlert(
        i18n.t("plugins.check_update"),
        i18n.t("plugins.updates_available", { count: updates.length }),
      );
    } else {
      showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.all_plugins_latest"));
    }
  } finally {
    checkingAllUpdates.value = false;
  }
}

function openRepository(url: string) {
  openUrl(url);
}

async function handleSaveSettings() {
  const errorMessage = await saveSettings();
  if (errorMessage) {
    showAlert(i18n.t("common.message_unknown_error"), errorMessage);
  }
}

function goToMarket() {
  showDependencyModal.value = false;
  router.push("/plugins?tab=market");
}
</script>

<template>
  <div class="plugins-view">
    <div class="plugins-toolbar">
      <div class="toolbar-left">
        <input
          v-model="searchQuery"
          type="text"
          class="plugin-search"
          :placeholder="i18n.t('plugins.search_placeholder')"
        />
      </div>
      <div class="toolbar-right">
        <SLButton :variant="batchMode ? 'primary' : 'secondary'" size="sm" @click="toggleBatchMode">
          {{ i18n.t("plugins.batch_mode") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="sm"
          :loading="checkingAllUpdates"
          @click="handleCheckAllUpdates"
        >
          {{ i18n.t("plugins.check_updates") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="sm"
          :loading="pluginStore.loading"
          @click="handleRefresh"
        >
          {{ i18n.t("plugins.refresh") }}
        </SLButton>
      </div>
    </div>

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

    <div v-if="pluginStore.error || installErrorMessage" class="error-banner">
      <span class="error-icon">!</span>
      <span class="error-text">{{ pluginStore.error || installErrorMessage }}</span>
    </div>

    <div v-if="pluginStore.loading && pluginStore.plugins.length === 0" class="loading-state">
      <div class="loading-spinner"></div>
      <span class="loading-text">{{ i18n.t("plugins.loading_plugins") }}</span>
    </div>

    <div v-else-if="!pluginStore.loading && pluginStore.plugins.length === 0" class="empty-state">
      <div class="empty-icon">
        <Layers :size="48" :stroke-width="1.5" />
      </div>
      <h3 class="empty-title">{{ i18n.t("plugins.no_plugins") }}</h3>
      <p class="empty-desc">{{ i18n.t("plugins.no_plugins_desc") }}</p>
    </div>

    <div v-else>
      <div v-if="batchMode" class="batch-action-bar">
        <div class="batch-action-left">
          <span class="selected-count">{{
            i18n.t("plugins.selected_count", { count: selectedPlugins.size })
          }}</span>
        </div>
        <div class="batch-action-right">
          <SLButton variant="secondary" size="sm" @click="selectAll(pluginStore.plugins)">
            {{ i18n.t("plugins.select_all") }}
          </SLButton>
          <SLButton variant="secondary" size="sm" @click="invertSelection(pluginStore.plugins)">
            {{ i18n.t("plugins.invert_selection") }}
          </SLButton>
          <SLButton variant="secondary" size="sm" @click="deselectAll">
            {{ i18n.t("plugins.deselect_all") }}
          </SLButton>
          <SLButton
            variant="danger"
            size="sm"
            :disabled="selectedPlugins.size === 0"
            @click="showBatchDeleteConfirm"
          >
            {{ i18n.t("plugins.batch_delete") }}
          </SLButton>
        </div>
      </div>
      <div class="plugin-grid">
        <SLCard
          v-for="plugin in filteredPlugins"
          :key="plugin.manifest.id"
          class="plugin-card"
          :class="{ 'plugin-card--selected': batchMode && selectedPlugins.has(plugin.manifest.id) }"
        >
          <div class="plugin-content">
            <label v-if="batchMode" class="plugin-checkbox" @click.stop>
              <SLCheckbox
                :modelValue="selectedPlugins.has(plugin.manifest.id)"
                @update:modelValue="togglePluginSelection(plugin.manifest.id)"
              />
            </label>

            <div class="plugin-card-actions">
              <div
                v-if="pluginStore.updates[plugin.manifest.id]"
                class="update-badge"
                :title="i18n.t('plugins.update_available')"
              >
                <ShieldAlert :size="12" />
              </div>

              <div
                v-if="hasMissingRequiredDependencies(plugin)"
                class="dependency-indicator dependency-indicator--required"
                :title="getDependencyTooltip(plugin)"
                @click.stop="showMissingDependenciesModal(plugin)"
              ></div>
              <div
                v-else-if="hasMissingOptionalDependencies(plugin)"
                class="dependency-indicator dependency-indicator--optional"
                :title="getDependencyTooltip(plugin)"
                @click.stop="showMissingDependenciesModal(plugin)"
              ></div>

              <PluginPermissionPanel
                :plugin-id="plugin.manifest.id"
                :permissions="plugin.manifest.permissions || []"
              />

              <SLMenu
                :items="getPluginMenuItems(plugin.manifest.id)"
                position="bottom-end"
                @select="handleMenuSelect($event, plugin.manifest.id)"
              >
                <SLButton variant="ghost" icon-only size="sm">
                  <MoreVertical :size="16" />
                </SLButton>
              </SLMenu>
            </div>
            <div class="plugin-main">
              <div class="plugin-icon">
                <img
                  v-if="pluginStore.icons[plugin.manifest.id]"
                  :src="pluginStore.icons[plugin.manifest.id]"
                  alt="plugin icon"
                  class="plugin-icon-img"
                />
                <Layers v-else :size="32" :stroke-width="1.5" class="plugin-icon-default" />
              </div>
              <div class="plugin-info">
                <div class="plugin-header">
                  <div class="plugin-title-row">
                    <h3 class="plugin-name">
                      {{ getLocalizedPluginName(plugin.manifest, i18n.getLocale()) }}
                    </h3>
                    <span class="plugin-version">v{{ plugin.manifest.version }}</span>
                  </div>
                  <div class="plugin-author-row">
                    <span v-if="plugin.manifest.author" class="plugin-author">
                      by {{ plugin.manifest.author.name }}
                    </span>
                    <SLButton
                      v-if="plugin.manifest.repository"
                      variant="ghost"
                      icon-only
                      size="sm"
                      @click.stop="openRepository(plugin.manifest.repository)"
                      :title="i18n.t('plugins.open_repository')"
                    >
                      <Github :size="14" />
                    </SLButton>
                  </div>
                </div>
                <p v-if="plugin.manifest.description" class="plugin-description">
                  {{ getLocalizedPluginDescription(plugin.manifest, i18n.getLocale()) }}
                </p>
                <p
                  v-if="typeof plugin.state === 'object' && 'error' in plugin.state"
                  class="plugin-error-message"
                >
                  {{ plugin.state.error }}
                </p>
              </div>
            </div>

            <div class="plugin-footer">
              <span
                class="plugin-status"
                :style="{ color: getStatusColor(plugin.state) }"
                :title="
                  typeof plugin.state === 'object' && 'error' in plugin.state
                    ? plugin.state.error
                    : undefined
                "
              >
                {{ getStatusLabel(plugin.state) }}
              </span>
              <div class="plugin-actions">
                <SLButton
                  v-if="hasSettings(plugin)"
                  variant="ghost"
                  icon-only
                  size="sm"
                  @click="openSettings(plugin)"
                  :title="i18n.t('plugins.settings')"
                >
                  <Settings :size="16" />
                </SLButton>
                <SLSwitch
                  v-if="!safeMode"
                  :modelValue="isPluginEnabled(plugin.state)"
                  :disabled="
                    hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)
                  "
                  :title="
                    hasMissingRequiredDependencies(plugin) && !isPluginEnabled(plugin.state)
                      ? i18n.t('plugins.missing_required_deps')
                      : ''
                  "
                  @update:modelValue="
                    handleToggle(plugin.manifest.id, isPluginEnabled(plugin.state))
                  "
                  size="sm"
                />
                <span v-else class="safe-mode-label">{{
                  i18n.t("plugins.safe_mode_disabled")
                }}</span>
              </div>
            </div>
          </div>
        </SLCard>
      </div>
    </div>

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
      @go-market="goToMarket"
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

.safe-mode-label {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  background-color: var(--sl-surface);
  padding: 0.25rem 0.5rem;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border-light);
  align-self: center;
}

.plugins-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.plugin-search {
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  width: 180px;
  transition: all var(--sl-transition-fast);
}

.plugin-search:focus {
  outline: none;
  border-color: var(--sl-primary);
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

.error-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 16px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
}

.error-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--sl-error);
  color: var(--sl-text-inverse);
  font-size: 12px;
  font-weight: 700;
}

.error-text {
  color: var(--sl-error);
  font-size: 14px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  border-radius: var(--sl-radius-md);
  text-align: center;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  margin-top: 16px;
  color: var(--sl-text-secondary);
  font-size: 14px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  border-radius: var(--sl-radius-md);
  text-align: center;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
}

.empty-icon {
  color: var(--sl-text-tertiary);
  margin-bottom: 16px;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  margin: 0;
  max-width: 320px;
}

.plugin-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sl-space-md);
}

@media (max-width: 1200px) {
  .plugin-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 700px) {
  .plugin-grid {
    grid-template-columns: 1fr;
  }
}

.plugin-card {
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  height: 100%;
}

.plugin-card:hover {
  transform: translateY(-2px);
}

.plugin-card--selected {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.batch-action-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  margin-bottom: 12px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-primary);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.batch-action-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.batch-action-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selected-count {
  font-size: 14px;
  color: var(--sl-text-primary);
  font-weight: 500;
}

.plugin-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 5;
  display: flex;
  align-items: center;
}

.plugin-content {
  padding: 8px;
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.plugin-main {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  flex: 1;
}

.plugin-icon {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.plugin-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-md);
}

.plugin-icon-default {
  color: var(--sl-text-tertiary);
}

.plugin-info {
  flex: 1;
  min-width: 0;
}

.plugin-header {
  margin-bottom: 4px;
}

.plugin-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.plugin-version {
  flex-shrink: 0;
  padding: 1px 5px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.plugin-author {
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.plugin-author-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 2px;
}

.repo-link-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--sl-radius-xs);
  color: var(--sl-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.repo-link-btn:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-primary);
}

.plugin-description {
  margin: 6px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.plugin-error-message {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--sl-error);
  line-height: 1.4;
  word-break: break-word;
}

.plugin-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 8px;
  border-top: 1px solid var(--sl-border);
  margin-top: auto;
}

.plugin-status {
  font-size: 12px;
  font-weight: 500;
}

.plugin-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.settings-btn:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.plugin-card-actions {
  position: absolute;
  top: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  z-index: 10;
}

.update-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: var(--sl-primary);
  border-radius: 50%;
  color: var(--sl-text-inverse);
}

.dependency-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  flex-shrink: 0;
}

.dependency-indicator:hover {
  transform: scale(1.3);
}

.dependency-indicator--required {
  background: #ef4444;
  box-shadow: 0 0 6px rgba(239, 68, 68, 0.5);
}

.dependency-indicator--required:hover {
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.7);
}

.dependency-indicator--optional {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
}

.dependency-indicator--optional:hover {
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.7);
}

.header-right {
  display: flex;
  gap: 8px;
}
</style>
