<script setup lang="ts">
import { i18n } from "@language";
import { RefreshCw, AlertCircle, Search, Puzzle, Globe } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import PluginMarketDetailDialog from "@components/views/plugins/PluginMarketDetailDialog.vue";
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

const MARKET_BASE_URL = "https://sealantern-studio.github.io/plugin-market";
</script>

<template>
  <div class="market-view animate-fade-in-up">
    <div
      v-if="installFeedback"
      :class="['install-feedback', `install-feedback--${installFeedback.type}`]"
    >
      <span>{{ installFeedback.message }}</span>
      <button class="install-feedback-close" @click="clearFeedback">x</button>
    </div>

    <div v-if="showUrlEditor" class="url-editor glass">
      <span class="url-editor-label">{{ i18n.t("market.source_url") }}</span>
      <input
        v-model="urlInput"
        type="url"
        class="url-editor-input"
        :placeholder="MARKET_BASE_URL"
        @keydown.enter="saveMarketUrl"
      />
      <button class="url-editor-btn" @click="saveMarketUrl">
        {{ i18n.t("market.source_save") }}
      </button>
      <button
        v-if="customMarketUrl"
        class="url-editor-btn url-editor-btn--reset"
        @click="resetMarketUrl"
      >
        {{ i18n.t("market.source_reset") }}
      </button>
      <p v-if="isUsingCustomMarket" class="url-editor-note">
        {{ i18n.t("market.source_current", { host: activeMarketHost }) }}
      </p>
    </div>

    <div v-if="showCustomSourceConfirm && pendingMarketSource" class="source-confirm glass">
      <p class="source-confirm-title">{{ i18n.t("market.source_confirm_title") }}</p>
      <p class="source-confirm-text">{{ i18n.t("market.source_confirm_desc") }}</p>
      <p class="source-confirm-url">{{ pendingMarketSource }}</p>
      <div class="source-confirm-actions">
        <button class="url-editor-btn" @click="confirmCustomMarketUrl">
          {{ i18n.t("market.source_save") }}
        </button>
        <button class="url-editor-btn url-editor-btn--reset" @click="cancelCustomMarketUrl">
          {{ i18n.t("common.cancel") }}
        </button>
      </div>
    </div>

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

    <div v-if="loading" class="market-loading">
      <div class="loading-spinner"></div>
      <span class="loading-text">{{ i18n.t("market.loading") }}</span>
    </div>

    <div v-else-if="error" class="market-error">
      <AlertCircle :size="48" :stroke-width="1.5" />
      <p class="error-title">{{ i18n.t("market.error_title") }}</p>
      <p class="error-detail">{{ error }}</p>
      <p v-if="marketErrorHint" class="error-hint">{{ marketErrorHint }}</p>
      <button class="retry-btn" @click="loadMarket">
        {{ i18n.t("market.retry") }}
      </button>
    </div>

    <div v-else-if="!filteredPlugins.length" class="market-empty">
      <Search :size="48" :stroke-width="1.5" />
      <p>{{ i18n.t("market.no_plugins") }}</p>
    </div>

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

.install-feedback {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--sl-radius-md);
  border: 1px solid transparent;
  white-space: pre-wrap;
  line-height: 1.45;
  font-size: 13px;
}

.install-feedback--success {
  color: #2e7d32;
  background: rgba(46, 125, 50, 0.12);
  border-color: rgba(46, 125, 50, 0.28);
}

.install-feedback--warning {
  color: #8a6d00;
  background: rgba(245, 158, 11, 0.14);
  border-color: rgba(245, 158, 11, 0.3);
}

.install-feedback--error {
  color: var(--sl-error);
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.25);
}

.install-feedback-close {
  border: none;
  background: transparent;
  color: inherit;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
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

.market-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  text-align: center;
  color: var(--sl-text-tertiary);
}

.market-error,
.market-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-2xl);
  text-align: center;
  color: var(--sl-text-tertiary);
}

.error-hint {
  margin-top: 8px;
  color: var(--sl-text-secondary);
  max-width: 640px;
  line-height: 1.5;
}

.url-editor,
.source-confirm {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-secondary);
}

.url-editor-label,
.source-confirm-title {
  color: var(--sl-text-primary);
  font-size: 13px;
  font-weight: 600;
}

.url-editor-input {
  flex: 1 1 320px;
  min-width: 220px;
  padding: 8px 10px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
}

.url-editor-note,
.source-confirm-text {
  flex-basis: 100%;
  margin: 0;
  color: var(--sl-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.source-confirm-url {
  flex-basis: 100%;
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 12px;
  word-break: break-all;
}

.source-confirm-actions {
  display: flex;
  gap: 8px;
}

.url-editor-btn {
  padding: 8px 12px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  cursor: pointer;
}

.url-editor-btn--reset {
  color: var(--sl-text-secondary);
}

.retry-btn {
  padding: 8px 14px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  cursor: pointer;
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
}

.error-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--sl-text-primary);
  margin: 16px 0 8px;
}

.error-detail {
  font-size: 14px;
  color: var(--sl-text-tertiary);
  margin: 0 0 16px;
}

.retry-btn {
  padding: 8px 24px;
  border-radius: var(--sl-radius-md);
  border: none;
  background: var(--sl-primary);
  color: white;
  cursor: pointer;
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

.refresh-btn.active {
  border-color: var(--sl-primary);
  color: var(--sl-primary);
}

.url-editor {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: var(--sl-radius-md);
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.url-editor-label {
  font-size: 13px;
  color: var(--sl-text-secondary);
  white-space: nowrap;
}

.url-editor-input {
  flex: 1;
  min-width: 200px;
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
  font-size: 13px;
}

.url-editor-input:focus {
  outline: none;
  border-color: var(--sl-primary);
}

.url-editor-btn {
  padding: 6px 14px;
  border-radius: var(--sl-radius-sm);
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.url-editor-btn:hover {
  opacity: 0.85;
}

.url-editor-btn--reset {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  border: 1px solid var(--sl-border);
}

.url-editor-btn--reset:hover {
  opacity: 1;
  border-color: var(--sl-error);
  color: var(--sl-error);
}
</style>
