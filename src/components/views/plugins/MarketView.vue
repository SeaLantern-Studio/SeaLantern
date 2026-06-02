<script setup lang="ts">
import { i18n } from "@language";
import { RefreshCw, Puzzle, Globe } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
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

    <div v-else class="market-grid">
      <SLCard
        v-for="plugin in filteredPlugins"
        :key="plugin.id"
        class="market-card"
        @click="showDetail(plugin)"
      >
        <div class="card-icon">
          <img
            v-if="getIconUrl(plugin)"
            :src="getIconUrl(plugin)!"
            :alt="resolveI18n(plugin.name)"
          />
          <Puzzle v-else :size="32" :stroke-width="1.5" />
        </div>
        <div class="card-info">
          <div class="card-header">
            <span class="card-name">{{ resolveI18n(plugin.name) }}</span>
            <span class="card-version">{{ plugin.version ? "v" + plugin.version : "" }}</span>
          </div>
          <span class="card-author">by {{ plugin.author?.name || "Unknown" }}</span>
          <p class="card-desc">{{ resolveI18n(plugin.description) }}</p>

          <div v-if="plugin.dependencies?.length" class="card-deps">
            <span class="deps-label">{{ i18n.t("market.requires") }}</span>
            <span class="deps-list">{{ plugin.dependencies.join(", ") }}</span>
          </div>
          <div class="card-footer">
            <div class="card-tags">
              <span v-for="tag in plugin.categories?.slice(0, 2)" :key="tag" class="card-tag">{{
                getCategoryLabel(tag)
              }}</span>
            </div>
            <button
              :class="[
                'install-btn',
                {
                  installed: isInstalled(plugin.id),
                  'is-enabled': isInstalledAndEnabled(plugin.id),
                },
              ]"
              :disabled="isInstalled(plugin.id) || installing === plugin.id"
              :title="
                isInstalledAndEnabled(plugin.id) ? i18n.t('market.plugin_running_warning') : ''
              "
              @click.stop="handleInstall(plugin)"
            >
              {{ getInstallButtonText(plugin.id) }}
            </button>
          </div>
        </div>
      </SLCard>
    </div>

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

.market-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sl-space-md);
}

@media (max-width: 1200px) {
  .market-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .market-grid {
    grid-template-columns: 1fr;
  }
}

.market-card {
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  display: flex;
  gap: var(--sl-space-lg);
  box-sizing: border-box;
  height: 100%;
}

.market-card:hover {
  border-color: var(--sl-border);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.card-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-text-tertiary);
}

.card-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-md);
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.card-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--sl-text-primary);
  word-wrap: break-word;
}

.card-version {
  padding: 2px 6px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.card-author {
  font-size: 12px;
  color: var(--sl-text-secondary);
  margin-bottom: 10px;
}

.card-desc {
  margin: 0 0 12px 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-wrap: break-word;
}

.card-deps {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 6px 0;
  font-size: 12px;
}

.deps-label {
  color: var(--sl-warning);
  font-weight: 500;
}

.deps-list {
  color: var(--sl-text-secondary);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 14px;
  padding-top: 8px;
  border-top: 1px solid var(--sl-border);
}

.card-tags {
  display: flex;
  gap: 6px;
}

.card-tag {
  padding: 2px 8px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.install-btn {
  padding: 6px 16px;
  border-radius: var(--sl-radius-sm);
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.install-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.install-btn:disabled {
  cursor: not-allowed;
}

.install-btn.installed {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
}

.install-btn.is-enabled {
  background: var(--sl-bg-tertiary);
  color: var(--sl-warning);
  font-size: 12px;
}
</style>
