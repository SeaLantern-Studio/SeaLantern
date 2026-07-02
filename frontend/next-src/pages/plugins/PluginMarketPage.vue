<script setup lang="ts">
import { i18n } from "@language";
import { RefreshCw, Globe } from "@lucide/vue";
import { SLTabBar } from "@components/common";
import PluginMarketGrid from "@components/plugin/market/PluginMarketGrid.vue";
import PluginMarketSourcePanel from "@components/plugin/market/PluginMarketSourcePanel.vue";
import PluginMarketStatePanel from "@components/plugin/market/PluginMarketStatePanel.vue";
import { usePluginMarketPage } from "./usePluginMarketPage";

const {
  loading,
  error,
  installFeedback,
  searchQuery,
  selectedTag,
  installing,
  showUrlEditor,
  customMarketUrl,
  urlInput,
  activeMarketHost,
  isUsingCustomMarket,
  pendingMarketSource,
  showCustomSourceConfirm,
  marketErrorHint,
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
  getCategoryLabel,
  getIconUrl,
  loadMarket,
  installPlugin,
  openPluginDetail,
} = usePluginMarketPage();
</script>

<template>
  <div class="plugin-market-page">
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
          class="plugin-market-page__search"
        />
        <button
          class="plugin-market-page__action-btn"
          :class="{ active: customMarketUrl }"
          @click="toggleUrlEditor"
          :title="i18n.t('market.custom_source')"
        >
          <Globe :size="14" />
        </button>
        <button
          class="plugin-market-page__action-btn"
          @click="loadMarket"
          :disabled="loading"
          :title="i18n.t('plugins.refresh')"
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
      @show-detail="openPluginDetail"
      @install="installPlugin"
    />
  </div>
</template>

<style scoped>
.plugin-market-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.plugin-market-page__search {
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  width: 180px;
  transition: all var(--sl-transition-fast);
}

.plugin-market-page__search:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.plugin-market-page__action-btn {
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

.plugin-market-page__action-btn:hover {
  color: var(--sl-text-primary);
  background: var(--sl-bg-tertiary);
}

.plugin-market-page__action-btn.active {
  color: var(--sl-primary);
}

.plugin-market-page__action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.plugin-market-page__action-btn .spin {
  animation: plugin-market-page-spin 1s linear infinite;
}

@keyframes plugin-market-page-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
