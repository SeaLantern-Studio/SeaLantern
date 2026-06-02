import { onMounted, ref } from "vue";
import {
  fetchMarketCategories,
  fetchMarketPluginDetail,
  fetchMarketPlugins,
  installFromMarket,
  type MarketPluginInfo,
} from "@api/plugin";
import {
  getMarketPermissionLevel,
  MARKET_BASE_URL,
  resolveMarketValue,
  type MarketFeedback,
  type MarketFeedbackType,
  type MarketPlugin,
} from "@components/views/plugins/pluginMarketShared";
import { usePluginMarketSource } from "@components/views/plugins/usePluginMarketSource";
import { usePluginMarketViewState } from "@components/views/plugins/usePluginMarketViewState";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";

export function usePluginMarket() {
  const pluginStore = usePluginStore();
  const loading = ref(true);
  const error = ref<string | null>(null);
  const installFeedback = ref<MarketFeedback | null>(null);
  const marketPlugins = ref<MarketPlugin[]>([]);
  const categories = ref<Record<string, Record<string, string> | string>>({});
  const searchQuery = ref("");
  const selectedTag = ref<string | null>(null);
  const installing = ref<string | null>(null);
  const selectedPlugin = ref<MarketPlugin | null>(null);
  const detailLoading = ref(false);
  const pluginDetail = ref<MarketPluginInfo | null>(null);

  const {
    showUrlEditor,
    customMarketUrl,
    urlInput,
    activeMarketUrl,
    activeMarketHost,
    isUsingCustomMarket,
    pendingMarketSource,
    showCustomSourceConfirm,
    saveMarketUrl,
    confirmCustomMarketUrl,
    cancelCustomMarketUrl,
    resetMarketUrl,
    toggleUrlEditor,
  } = usePluginMarketSource({
    onSourceChanged: () => {
      void loadMarket();
    },
    showFeedback,
  });

  const {
    marketErrorHint,
    filteredPlugins,
    allTags,
    tagTabs,
    getCategoryLabel,
    getIconUrl,
    getMarketRequestUrl,
  } = usePluginMarketViewState({
    error,
    marketPlugins,
    categories,
    searchQuery,
    selectedTag,
    activeMarketUrl,
  });

  function normalizeErrorMessage(err: unknown): string {
    const normalized = normalizeAppError(err);
    return normalized.message;
  }

  function showFeedback(type: MarketFeedbackType, message: string, duration = 6000) {
    installFeedback.value = { type, message };
    if (duration > 0) {
      setTimeout(() => {
        if (installFeedback.value?.message === message) {
          installFeedback.value = null;
        }
      }, duration);
    }
  }

  function clearFeedback() {
    installFeedback.value = null;
  }

  function isInstalled(pluginId: string): boolean {
    return pluginStore.plugins.some((plugin) => plugin.manifest.id === pluginId);
  }

  function isInstalledAndEnabled(pluginId: string): boolean {
    const plugin = pluginStore.plugins.find((item) => item.manifest.id === pluginId);
    return !!plugin && plugin.state === "enabled";
  }

  function getInstallButtonText(pluginId: string): string {
    if (installing.value === pluginId) {
      return i18n.t("market.installing");
    }
    if (isInstalledAndEnabled(pluginId)) {
      return i18n.t("market.running_need_disable");
    }
    if (isInstalled(pluginId)) {
      return i18n.t("market.installed");
    }
    return i18n.t("market.install");
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

  async function loadMarket() {
    loading.value = true;
    error.value = null;

    try {
      const requestUrl = getMarketRequestUrl();
      const sourceUrl = activeMarketUrl.value.trim().replace(/\/$/, "") || MARKET_BASE_URL;
      const [plugins, categoryMap] = await Promise.all([
        fetchMarketPlugins(requestUrl),
        fetchMarketCategories(requestUrl).catch(() => ({})),
      ]);
      marketPlugins.value = plugins;
      categories.value = categoryMap;
      pluginLogger.info("Market", "插件市场列表已更新", {
        source: sourceUrl,
        pluginCount: plugins.length,
        categoryCount: Object.keys(categoryMap).length,
      });
    } catch (err) {
      const normalized = normalizeAppError(err);
      error.value = normalized.message;
      pluginLogger.error("Market", "插件市场列表读取失败", normalized);
    } finally {
      loading.value = false;
    }
  }

  async function showDetail(plugin: MarketPlugin) {
    selectedPlugin.value = plugin;
    detailLoading.value = true;

    try {
      const requestUrl = getMarketRequestUrl();
      const sourceUrl = activeMarketUrl.value.trim().replace(/\/$/, "") || MARKET_BASE_URL;
      pluginDetail.value = await fetchMarketPluginDetail(
        plugin._path || `plugins/${plugin.id}.json`,
        requestUrl,
      );
      pluginLogger.info("Market", "插件详情已读取", {
        pluginId: plugin.id,
        source: sourceUrl,
      });
    } catch (err) {
      pluginDetail.value = null;
      pluginLogger.error("Market", `插件详情读取失败: ${plugin.id}`, normalizeAppError(err));
    } finally {
      detailLoading.value = false;
    }
  }

  function closeDetail() {
    selectedPlugin.value = null;
    pluginDetail.value = null;
  }

  async function handleInstall(plugin: MarketPlugin) {
    if (installing.value || isInstalled(plugin.id)) {
      return;
    }

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
        showFeedback("warning", i18n.t("market.untrusted_download_warning"));
      } else {
        showFeedback("success", i18n.t("market.install_success"));
      }

      pluginLogger.info("Market", "插件市场安装完成", {
        pluginId: plugin.id,
        untrustedUrl: Boolean(result?.untrusted_url),
      });
    } catch (err) {
      const errorMessage = normalizeErrorMessage(err);
      const hint = marketErrorHint.value || undefined;
      const extraHint = hint ? `\n${hint}` : "";
      showFeedback("error", `${i18n.t("market.install_failed")}: ${errorMessage}${extraHint}`, 0);
      pluginLogger.error("Market", `插件市场安装失败: ${plugin.id}`, normalizeAppError(err));
    } finally {
      installing.value = null;
    }
  }

  onMounted(() => {
    void loadMarket();
  });

  return {
    loading,
    error,
    installFeedback,
    marketPlugins,
    categories,
    searchQuery,
    selectedTag,
    installing,
    selectedPlugin,
    detailLoading,
    pluginDetail,
    showUrlEditor,
    customMarketUrl,
    urlInput,
    activeMarketUrl,
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
    resolveI18n: resolveMarketValue,
    isInstalled,
    isInstalledAndEnabled,
    getInstallButtonText,
    getPermissionLevel: getMarketPermissionLevel,
    getPermissionLabel,
    getPermissionDesc,
    getCategoryLabel,
    getIconUrl,
    loadMarket,
    showDetail,
    handleInstall,
    closeDetail,
  };
}
