<script setup lang="ts">
import { i18n } from "@language";
import { RefreshCw, Globe } from "@lucide/vue";
import PluginMarketGrid from "@components/views/plugins/PluginMarketGrid.vue";
import PluginMarketDetailDialog from "@components/views/plugins/PluginMarketDetailDialog.vue";
import PluginMarketSourcePanel from "@components/views/plugins/PluginMarketSourcePanel.vue";
import PluginMarketStatePanel from "@components/views/plugins/PluginMarketStatePanel.vue";
import { usePluginMarket } from "@components/views/plugins/usePluginMarket";
import { SLTabBar } from "@components/common";
const {
  loading,
  error,
  installFeedback,
  searchQuery,
  selectedTag,
  installing,
  selectedPlugin,
  detailLoading,
  pluginDetail,
  showUrlEditor,
  customMarketUrl,
  urlInput,
  activeMarketHost,
  isUsingCustomMarket,
  marketErrorHint,
  pendingMarketSource,
  showCustomSourceConfirm,
  filteredPlugins,
  allTags,
  tagTabs,
  clearFeedback,
  saveMarketUrl,
  confirmCustomMarketUrl,
  cancelCustomMarketUrl,
  resetMarketUrl,
  toggleUrlEditor,
  resolveI18n,
  isInstalled,
  isInstalledAndEnabled,
  getInstallButtonText,
  getPermissionLevel,
  getPermissionLabel,
  getPermissionDesc,
  getCategoryLabel,
  getIconUrl,
  loadMarket,
  showDetail,
  handleInstall,
  closeDetail,
} = usePluginMarket();
</script>

<template>
  <div class="market-view animate-fade-in-up">
    <PluginMarketSourcePanel
      :install-feedback="installFeedback"
      :show-url-editor="showUrlEditor"
      :custom-market-url="customMarketUrl"
      :url-input="urlInput"
      :is-using-custom-market="isUsingCustomMarket"
      :active-market-host="activeMarketHost"
      :pending-market-source="pendingMarketSource"
      :show-custom-source-confirm="showCustomSourceConfirm"
      @clear-feedback="clearFeedback"
      @update:url-input="urlInput = $event"
      @save-market-url="saveMarketUrl"
      @reset-market-url="resetMarketUrl"
      @confirm-custom-source="confirmCustomMarketUrl"
      @cancel-custom-source="cancelCustomMarketUrl"
    />

    <SLTabBar v-if="allTags.length" v-model="selectedTag" :tabs="tagTabs" :level="2">
      <template #extra>
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="i18n.t('market.search_placeholder')"
          class="market-search"
        />
        <button
          class="action-btn"
          :class="{ active: customMarketUrl }"
          @click="toggleUrlEditor"
          :title="i18n.t('market.custom_source')"
        >
          <Globe :size="14" />
        </button>
        <button
          class="action-btn"
          @click="loadMarket"
          :disabled="loading"
          :title="i18n.t('market.refresh')"
        >
          <RefreshCw :size="14" :class="{ spin: loading }" />
        </button>
      </template>
    </SLTabBar>

    <PluginMarketStatePanel
      v-if="loading || error || !filteredPlugins.length"
      :loading="loading"
      :error="error"
      :market-error-hint="marketErrorHint"
      :has-plugins="filteredPlugins.length > 0"
      @retry="loadMarket"
    />

    <PluginMarketGrid
      v-else
      :plugins="filteredPlugins"
      :installing-plugin-id="installing"
      :get-icon-url="getIconUrl"
      :resolve-i18n="resolveI18n"
      :get-category-label="getCategoryLabel"
      :is-installed="isInstalled"
      :is-installed-and-enabled="isInstalledAndEnabled"
      :get-install-button-text="getInstallButtonText"
      @show-detail="showDetail"
      @install="handleInstall"
    />

    <PluginMarketDetailDialog
      :plugin="selectedPlugin"
      :plugin-detail="pluginDetail"
      :detail-loading="detailLoading"
      :installing-plugin-id="installing"
      :get-icon-url="getIconUrl"
      :resolve-i18n="resolveI18n"
      :get-install-button-text="getInstallButtonText"
      :is-installed="isInstalled"
      :is-installed-and-enabled="isInstalledAndEnabled"
      :get-permission-level="getPermissionLevel"
      :get-permission-label="getPermissionLabel"
      :get-permission-desc="getPermissionDesc"
      @close="closeDetail"
      @install="handleInstall"
    />
  </div>
</template>

<style scoped>
.market-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  min-height: 100%;
  flex: 1;
}

.market-search {
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  width: 180px;
  transition: all var(--sl-transition-fast);
}

.market-search:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.action-btn {
  padding: 6px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid transparent;
  background: transparent;
  color: var(--sl-text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--sl-transition-fast);
}

.action-btn:hover {
  color: var(--sl-text-primary);
  background: var(--sl-bg-tertiary);
}

.action-btn.active {
  color: var(--sl-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn .spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
