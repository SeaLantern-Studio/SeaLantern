<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import {
  fetchMarketPlugins,
  fetchMarketPluginDetail,
  fetchMarketCategories,
  installFromMarket,
} from "@api/plugin";
import type { MarketPluginInfo } from "@api/plugin";
import { i18n } from "@language";
import { RefreshCw, AlertCircle, Search, Puzzle, X, Globe } from "lucide-vue-next";
import SLCard from "@components/common/SLCard.vue";
import { SLTabBar, type TabBarItem } from "@components/common";

type MarketPlugin = MarketPluginInfo & { _path?: string };

const MARKET_BASE_URL = "https://sealantern-studio.github.io/plugin-market";
const MARKET_URL_KEY = "sealantern_market_url";

const pluginStore = usePluginStore();
const loading = ref(true);
const error = ref<string | null>(null);
const installFeedback = ref<{
  type: "success" | "warning" | "error";
  message: string;
} | null>(null);
const marketPlugins = ref<MarketPlugin[]>([]);
const categories = ref<Record<string, Record<string, string> | string>>({});
const searchQuery = ref("");
const selectedTag = ref<string | null>(null);
const installing = ref<string | null>(null);
const selectedPlugin = ref<MarketPlugin | null>(null);
const detailLoading = ref(false);
const pluginDetail = ref<MarketPluginInfo | null>(null);
const showUrlEditor = ref(false);
const customMarketUrl = ref(localStorage.getItem(MARKET_URL_KEY) || "");
const urlInput = ref(customMarketUrl.value);

const activeMarketUrl = computed(() => customMarketUrl.value.trim() || MARKET_BASE_URL);
const marketErrorHint = computed<string>(() => {
  if (!error.value) return "";
  return resolveMarketNetworkHint(error.value);
});

function showInstallFeedback(
  type: "success" | "warning" | "error",
  message: string,
  duration = 6000,
) {
  installFeedback.value = { type, message };
  if (duration > 0) {
    setTimeout(() => {
      if (installFeedback.value?.message === message) {
        installFeedback.value = null;
      }
    }, duration);
  }
}

function saveMarketUrl() {
  const url = urlInput.value.trim();
  customMarketUrl.value = url;
  if (url) {
    localStorage.setItem(MARKET_URL_KEY, url);
  } else {
    localStorage.removeItem(MARKET_URL_KEY);
  }
  showUrlEditor.value = false;
  loadMarket();
}

function resetMarketUrl() {
  urlInput.value = "";
  customMarketUrl.value = "";
  localStorage.removeItem(MARKET_URL_KEY);
  showUrlEditor.value = false;
  loadMarket();
}

const filteredPlugins = computed(() => {
  let result = marketPlugins.value;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(
      (p) =>
        resolveI18n(p.name).toLowerCase().includes(q) ||
        resolveI18n(p.description).toLowerCase().includes(q) ||
        p.author?.name?.toLowerCase().includes(q),
    );
  }
  if (selectedTag.value) {
    result = result.filter((p) => p.categories?.includes(selectedTag.value!));
  }
  return result;
});

const allTags = computed(() => {
  const tags = new Set<string>();
  marketPlugins.value.forEach((p) => p.categories?.forEach((t) => tags.add(t)));
  return Array.from(tags);
});

function resolveI18n(val: Record<string, string> | string | undefined): string {
  if (!val) return "";
  if (typeof val === "string") return val;
  const lang = navigator.language || "zh-CN";
  const key = lang.startsWith("zh") ? "zh-CN" : "en-US";
  return val[key] || val["zh-CN"] || Object.values(val)[0] || "";
}

function isInstalled(pluginId: string): boolean {
  return pluginStore.plugins.some((p) => p.manifest.id === pluginId);
}

function isInstalledAndEnabled(pluginId: string): boolean {
  const plugin = pluginStore.plugins.find((p) => p.manifest.id === pluginId);
  return !!plugin && plugin.state === "enabled";
}

function getInstallButtonText(pluginId: string): string {
  if (installing.value === pluginId) return i18n.t("market.installing");
  if (isInstalledAndEnabled(pluginId)) return i18n.t("market.running_need_disable");
  if (isInstalled(pluginId)) return i18n.t("market.installed");
  return i18n.t("market.install");
}

const CRITICAL_PERMS = new Set(["execute_program", "plugin_folder_access"]);
const DANGEROUS_PERMS = new Set(["fs", "network", "server", "console"]);

function getPermissionLevel(perm: string): "critical" | "dangerous" | "normal" {
  if (CRITICAL_PERMS.has(perm)) return "critical";
  if (DANGEROUS_PERMS.has(perm)) return "dangerous";
  return "normal";
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

function getCategoryLabel(key: string): string {
  const lang = navigator.language || "zh-CN";
  const langKey = lang.startsWith("zh") ? "zh-CN" : "en-US";
  const cat = categories.value[key];
  if (!cat) return key;
  if (typeof cat === "string") return cat;
  return cat[langKey] || cat["zh-CN"] || key;
}

function getIconUrl(plugin: MarketPlugin): string | null {
  if (!plugin.icon_url || !plugin._path) return null;
  const dir = plugin._path.replace(/\/[^/]+$/, "");
  return `${activeMarketUrl.value.trim().replace(/\/$/, "")}/${dir}/${plugin.icon_url}`;
}

function normalizeErrorMessage(err: unknown): string {
  if (err instanceof Error && err.message) return err.message;
  if (typeof err === "string") return err;
  return String(err);
}

function resolveMarketNetworkHint(message: string): string {
  const text = message.toLowerCase();
  const looksLikeNetworkIssue =
    text.includes("download") ||
    text.includes("fetch") ||
    text.includes("network") ||
    text.includes("timeout") ||
    text.includes("proxy") ||
    text.includes("连接") ||
    text.includes("请求") ||
    text.includes("下载");

  if (!looksLikeNetworkIssue) {
    return "";
  }

  const isProxyRefused =
    text.includes("127.0.0.1:9") ||
    text.includes("actively refused") ||
    text.includes("connection refused") ||
    text.includes("proxyconnect") ||
    text.includes("proxy connect") ||
    text.includes("无法连接") ||
    text.includes("积极拒绝");

  if (isProxyRefused) {
    return i18n.t("market.network_hint_proxy");
  }
  if (text.includes("timed out") || text.includes("timeout") || text.includes("超时")) {
    return i18n.t("market.network_hint_timeout");
  }
  return i18n.t("market.network_hint_check");
}

async function loadMarket() {
  loading.value = true;
  error.value = null;
  try {
    const url = activeMarketUrl.value.trim().replace(/\/$/, "");
    const [plugins, cats] = await Promise.all([
      fetchMarketPlugins(url === MARKET_BASE_URL ? undefined : url),
      fetchMarketCategories(url === MARKET_BASE_URL ? undefined : url).catch(() => ({})),
    ]);
    marketPlugins.value = plugins;
    categories.value = cats;
  } catch (e: any) {
    error.value = normalizeErrorMessage(e);
  } finally {
    loading.value = false;
  }
}

async function showDetail(plugin: MarketPlugin) {
  selectedPlugin.value = plugin;
  detailLoading.value = true;
  try {
    const url = activeMarketUrl.value.trim().replace(/\/$/, "");
    pluginDetail.value = await fetchMarketPluginDetail(
      plugin._path || `plugins/${plugin.id}.json`,
      url === MARKET_BASE_URL ? undefined : url,
    );
  } catch {
    pluginDetail.value = null;
  } finally {
    detailLoading.value = false;
  }
}

async function handleInstall(plugin: MarketPlugin) {
  if (installing.value || isInstalled(plugin.id)) return;
  installing.value = plugin.id;
  try {
    const result = await installFromMarket({
      pluginId: plugin.id,
      downloadUrl: plugin.download_url,
      repo: plugin.repo,
      downloadType: plugin.download_type,
      releaseAsset: plugin.release_asset,
      branch: plugin.branch,
    });
    await pluginStore.loadPlugins();
    if (result?.untrusted_url) {
      showInstallFeedback("warning", i18n.t("market.untrusted_download_warning"));
    } else {
      showInstallFeedback("success", i18n.t("market.install_success"));
    }
  } catch (e: any) {
    const errorMessage = normalizeErrorMessage(e);
    const hint = resolveMarketNetworkHint(errorMessage);
    const extraHint = hint ? `\n${hint}` : "";
    showInstallFeedback(
      "error",
      `${i18n.t("market.install_failed")}: ${errorMessage}${extraHint}`,
      0,
    );
  } finally {
    installing.value = null;
  }
}

function closeDetail() {
  selectedPlugin.value = null;
  pluginDetail.value = null;
}

onMounted(() => {
  loadMarket();
});
</script>

<template>
  <div class="market-view animate-fade-in-up">
    <div
      v-if="installFeedback"
      :class="['install-feedback', `install-feedback--${installFeedback.type}`]"
    >
      <span>{{ installFeedback.message }}</span>
      <button class="install-feedback-close" @click="installFeedback = null">x</button>
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
    </div>

    <SLTabBar
      v-if="allTags.length"
      v-model="selectedTag"
      :tabs="[
        { key: null, label: i18n.t('config.categories.all') },
        ...allTags.map((tag) => ({ key: tag, label: getCategoryLabel(tag) })),
      ]"
      :level="2"
    >
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
          @click="showUrlEditor = !showUrlEditor"
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

    <Teleport to="body">
      <div v-if="selectedPlugin" class="modal-overlay" @click.self="closeDetail">
        <div class="detail-modal glass-strong">
          <button class="modal-close" @click="closeDetail">
            <X :size="20" />
          </button>
          <div class="detail-header">
            <div class="detail-icon">
              <img
                v-if="getIconUrl(selectedPlugin)"
                :src="getIconUrl(selectedPlugin)!"
                :alt="resolveI18n(selectedPlugin.name)"
              />
              <Puzzle v-else :size="48" :stroke-width="1.5" />
            </div>
            <div class="detail-title">
              <h2>{{ resolveI18n(selectedPlugin.name) }}</h2>
              <span class="detail-version">{{
                selectedPlugin.version ? "v" + selectedPlugin.version : ""
              }}</span>
              <span class="detail-author">by {{ selectedPlugin.author?.name }}</span>
            </div>
          </div>
          <div v-if="detailLoading" class="detail-loading">
            <div class="loading-spinner"></div>
          </div>
          <div v-else class="detail-body">
            <p class="detail-desc">
              {{ resolveI18n(pluginDetail?.description || selectedPlugin.description) }}
            </p>
            <div v-if="pluginDetail?.permissions?.length" class="detail-section">
              <h3>{{ i18n.t("market.permissions") }}</h3>
              <div class="permission-badges">
                <span
                  v-for="perm in pluginDetail.permissions"
                  :key="perm"
                  :class="['perm-badge', `perm-badge--${getPermissionLevel(perm)}`]"
                  :title="getPermissionDesc(perm)"
                  >{{ getPermissionLabel(perm) }}</span
                >
              </div>
            </div>
            <div v-if="pluginDetail?.changelog" class="detail-section">
              <h3>{{ i18n.t("market.changelog") }}</h3>
              <pre class="changelog">{{ pluginDetail.changelog }}</pre>
            </div>
          </div>
          <div class="detail-footer">
            <button
              :class="[
                'install-btn-lg',
                {
                  installed: isInstalled(selectedPlugin.id),
                  'is-enabled': isInstalledAndEnabled(selectedPlugin.id),
                },
              ]"
              :disabled="isInstalled(selectedPlugin.id) || installing === selectedPlugin.id"
              :title="
                isInstalledAndEnabled(selectedPlugin.id)
                  ? i18n.t('market.plugin_running_warning')
                  : ''
              "
              @click="handleInstall(selectedPlugin)"
            >
              {{ getInstallButtonText(selectedPlugin.id) }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
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

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.detail-modal {
  width: 90%;
  max-width: 560px;
  max-height: 80vh;
  overflow-y: auto;
  border-radius: var(--sl-radius-lg);
  padding: 24px;
  position: relative;
}

.modal-close {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  border-radius: var(--sl-radius-md);
}

.modal-close:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.detail-header {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
}

.detail-icon {
  flex-shrink: 0;
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-text-tertiary);
}

.detail-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-lg);
}

.detail-title h2 {
  margin: 0;
  font-size: 20px;
  color: var(--sl-text-primary);
}

.detail-version {
  display: inline-block;
  padding: 2px 8px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 12px;
  color: var(--sl-text-tertiary);
  margin-top: 4px;
}

.detail-author {
  display: block;
  font-size: 13px;
  color: var(--sl-text-secondary);
  margin-top: 4px;
}

.detail-loading {
  display: flex;
  justify-content: center;
  padding: 32px;
}

.detail-body {
  margin-bottom: 20px;
}

.detail-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.detail-section {
  margin-top: 16px;
}

.detail-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px;
}

.permission-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-badge {
  padding: 3px 10px;
  border-radius: var(--sl-radius-lg);
  font-size: 12px;
  font-weight: 500;
  cursor: default;
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  border: 1px solid var(--sl-border);
}

.perm-badge--dangerous {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.perm-badge--critical {
  background: rgba(239, 68, 68, 0.12);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.changelog {
  margin: 0;
  padding: 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-md);
  font-size: 12px;
  color: var(--sl-text-secondary);
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}

.detail-footer {
  display: flex;
  justify-content: flex-end;
}

.install-btn-lg {
  padding: 10px 32px;
  border-radius: 8px;
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.install-btn-lg:hover:not(:disabled) {
  opacity: 0.9;
}

.install-btn-lg:disabled {
  cursor: not-allowed;
}

.install-btn-lg.installed {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
}

.install-btn-lg.is-enabled {
  background: var(--sl-bg-tertiary);
  color: var(--sl-warning);
  font-size: 13px;
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
