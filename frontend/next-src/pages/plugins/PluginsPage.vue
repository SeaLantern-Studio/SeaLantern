<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import PluginBatchResultModal from "@components/plugin/installer/PluginBatchResultModal.vue";
import PluginChooserDialog from "@components/plugin/installer/PluginChooserDialog.vue";
import PluginDependencyPromptModal from "@components/plugin/installer/PluginDependencyPromptModal.vue";
import SLPermissionDialog from "@components/plugin/SLPermissionDialog.vue";
import { i18n } from "@language";
import InstalledPluginCard from "@next-src/components/plugins/InstalledPluginCard.vue";
import PluginsEmptyState from "@next-src/components/plugins/PluginsEmptyState.vue";
import PluginsSummaryBar from "@next-src/components/plugins/PluginsSummaryBar.vue";
import { usePluginsPage } from "./usePluginsPage";

const {
  pluginStore,
  installer,
  installedPlugins,
  totalCount,
  enabledCount,
  updateCount,
  hasPlugins,
  isBootstrapping,
  isRefreshing,
  checkingUpdates,
  errorMessage,
  permissionDialogOpen,
  pendingPermissionPluginName,
  pendingPermissionList,
  permissionDialogTitle,
  permissionDialogMessage,
  refreshPlugins,
  checkAllUpdates,
  clearError,
  getPluginName,
  getPluginDescription,
  getDependencyDisplayName,
  getPluginAuthor,
  getPluginMeta,
  getPluginStateLabel,
  getPluginStateTone,
  getMissingRequiredDependencies,
  getMissingOptionalDependencies,
  getUpdateSummary,
  canOpenDetails,
  openPluginDetails,
  openPluginMarket,
  openRepository,
  togglePlugin,
  closePermissionDialog,
  confirmEnablePlugin,
} = usePluginsPage();

const {
  chooserOpen,
  showDependencyModal,
  installedPluginName,
  missingDependencies,
  showBatchResultModal,
  batchInstallResult,
  openChooser,
  pickFile,
  pickFolder,
} = installer;
</script>

<template>
  <div class="plugins-page">
    <PluginsSummaryBar
      :total-count="totalCount"
      :enabled-count="enabledCount"
      :update-count="updateCount"
      :refreshing="isRefreshing"
      :checking-updates="checkingUpdates"
      @import-local="openChooser()"
      @refresh="refreshPlugins()"
      @check-updates="checkAllUpdates()"
      @open-market="openPluginMarket()"
    />

    <PluginChooserDialog
      :open="chooserOpen"
      @update:open="chooserOpen = $event"
      @pick-file="pickFile"
      @pick-folder="pickFolder"
    />

    <section v-if="errorMessage" class="plugins-page__error-banner" role="alert" aria-live="polite">
      <div class="plugins-page__error-copy">
        <strong>{{ i18n.t("plugins.next.error_title") }}</strong>
        <span>{{ errorMessage }}</span>
      </div>

      <div class="plugins-page__error-actions">
        <SLButton variant="ghost" size="sm" @click="clearError()">{{
          i18n.t("plugins.next.close_error")
        }}</SLButton>
        <SLButton variant="secondary" size="sm" :loading="isRefreshing" @click="refreshPlugins()">
          {{ i18n.t("plugins.next.retry") }}
        </SLButton>
      </div>
    </section>

    <SLCard
      v-if="isBootstrapping && !hasPlugins"
      variant="outline"
      class="plugins-page__loading-card"
    >
      <div class="plugins-page__loading-state">
        <div class="plugins-page__spinner" aria-hidden="true"></div>
        <div>
          <strong class="plugins-page__loading-title">{{
            i18n.t("plugins.next.loading_title")
          }}</strong>
          <p class="plugins-page__loading-description">
            {{ i18n.t("plugins.next.loading_description") }}
          </p>
        </div>
      </div>
    </SLCard>

    <PluginsEmptyState
      v-else-if="!hasPlugins"
      :loading="isRefreshing"
      @refresh="refreshPlugins()"
      @open-market="openPluginMarket()"
    />

    <section v-else class="plugins-page__list">
      <InstalledPluginCard
        v-for="plugin in installedPlugins"
        :key="plugin.manifest.id"
        :plugin="plugin"
        :display-name="getPluginName(plugin)"
        :description="getPluginDescription(plugin)"
        :author-name="getPluginAuthor(plugin)"
        :meta-items="getPluginMeta(plugin)"
        :state-label="getPluginStateLabel(plugin.state)"
        :state-tone="getPluginStateTone(plugin.state)"
        :icon-url="pluginStore.icons[plugin.manifest.id]"
        :enabled="plugin.state === 'enabled'"
        :can-toggle="plugin.actions.can_toggle"
        :has-missing-required-dependencies="getMissingRequiredDependencies(plugin).length > 0"
        :has-missing-optional-dependencies="getMissingOptionalDependencies(plugin).length > 0"
        :update-summary="getUpdateSummary(plugin)"
        :can-open-details="canOpenDetails(plugin)"
        @toggle="togglePlugin"
        @open-details="openPluginDetails"
        @open-repository="openRepository"
      />
    </section>

    <SLPermissionDialog
      :show="permissionDialogOpen"
      :plugin-name="pendingPermissionPluginName"
      :permissions="pendingPermissionList"
      :title="permissionDialogTitle"
      :message="permissionDialogMessage"
      @confirm="confirmEnablePlugin"
      @cancel="closePermissionDialog"
    />

    <PluginDependencyPromptModal
      :visible="showDependencyModal"
      :installed-plugin-name="installedPluginName"
      :missing-dependencies="missingDependencies"
      :get-dep-display-name="getDependencyDisplayName"
      @close="showDependencyModal = false"
      @go-market="openPluginMarket()"
    />

    <PluginBatchResultModal
      :visible="showBatchResultModal"
      :batch-install-result="batchInstallResult"
      @close="showBatchResultModal = false"
    />
  </div>
</template>

<style scoped>
.plugins-page {
  min-width: 0;
  display: grid;
  gap: 18px;
}

.plugins-page__error-banner {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid rgba(239, 68, 68, 0.22);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.plugins-page__error-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.plugins-page__error-copy strong {
  font-size: 0.95rem;
}

.plugins-page__error-copy span {
  line-height: 1.5;
  word-break: break-word;
}

.plugins-page__error-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.plugins-page__loading-card {
  padding: 4px;
}

.plugins-page__loading-state {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 20px;
}

.plugins-page__spinner {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 3px solid color-mix(in srgb, var(--sl-primary) 18%, transparent);
  border-top-color: var(--sl-primary);
  animation: plugins-page-spin 0.9s linear infinite;
}

.plugins-page__loading-title {
  display: block;
  margin-bottom: 4px;
  color: var(--sl-text-primary);
}

.plugins-page__loading-description {
  margin: 0;
  color: var(--sl-text-secondary);
}

.plugins-page__list {
  display: grid;
  gap: 16px;
}

@keyframes plugins-page-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 720px) {
  .plugins-page__error-banner,
  .plugins-page__loading-state {
    flex-direction: column;
  }
}
</style>
