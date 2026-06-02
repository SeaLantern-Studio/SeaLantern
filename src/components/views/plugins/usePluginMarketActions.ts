import {
  fetchMarketCategories,
  fetchMarketPluginDetail,
  fetchMarketPlugins,
  installFromMarket,
  type MarketPluginInfo,
} from "@api/plugin";
import { MARKET_BASE_URL, type MarketPlugin } from "./pluginMarketShared";
import { i18n } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";
import type { Ref } from "vue";

interface UsePluginMarketActionsOptions {
  loading: Ref<boolean>;
  error: Ref<string | null>;
  marketPlugins: Ref<MarketPlugin[]>;
  categories: Ref<Record<string, Record<string, string> | string>>;
  installing: Ref<string | null>;
  selectedPlugin: Ref<MarketPlugin | null>;
  detailLoading: Ref<boolean>;
  pluginDetail: Ref<MarketPluginInfo | null>;
  marketErrorHint: Ref<string>;
  getMarketRequestUrl: () => string | undefined;
  getSourceUrl: () => string;
  loadPlugins: () => Promise<void>;
  isInstalled: (pluginId: string) => boolean;
  showFeedback: (type: "success" | "warning" | "error", message: string, duration?: number) => void;
}

function normalizeErrorMessage(err: unknown): string {
  const normalized = normalizeAppError(err);
  return normalized.message;
}

export function usePluginMarketActions(options: UsePluginMarketActionsOptions) {
  async function loadMarket() {
    options.loading.value = true;
    options.error.value = null;

    try {
      const requestUrl = options.getMarketRequestUrl();
      const sourceUrl = options.getSourceUrl() || MARKET_BASE_URL;
      const [plugins, categoryMap] = await Promise.all([
        fetchMarketPlugins(requestUrl),
        fetchMarketCategories(requestUrl).catch(() => ({})),
      ]);
      options.marketPlugins.value = plugins;
      options.categories.value = categoryMap;
      pluginLogger.info("Market", "插件市场列表已更新", {
        source: sourceUrl,
        pluginCount: plugins.length,
        categoryCount: Object.keys(categoryMap).length,
      });
    } catch (err) {
      const normalized = normalizeAppError(err);
      options.error.value = normalized.message;
      pluginLogger.error("Market", "插件市场列表读取失败", normalized);
    } finally {
      options.loading.value = false;
    }
  }

  async function showDetail(plugin: MarketPlugin) {
    options.selectedPlugin.value = plugin;
    options.detailLoading.value = true;

    try {
      const requestUrl = options.getMarketRequestUrl();
      const sourceUrl = options.getSourceUrl() || MARKET_BASE_URL;
      options.pluginDetail.value = await fetchMarketPluginDetail(
        plugin._path || `plugins/${plugin.id}.json`,
        requestUrl,
      );
      pluginLogger.info("Market", "插件详情已读取", {
        pluginId: plugin.id,
        source: sourceUrl,
      });
    } catch (err) {
      options.pluginDetail.value = null;
      pluginLogger.error("Market", `插件详情读取失败: ${plugin.id}`, normalizeAppError(err));
    } finally {
      options.detailLoading.value = false;
    }
  }

  function closeDetail() {
    options.selectedPlugin.value = null;
    options.pluginDetail.value = null;
  }

  async function handleInstall(plugin: MarketPlugin) {
    if (options.installing.value || options.isInstalled(plugin.id)) {
      return;
    }

    options.installing.value = plugin.id;
    try {
      const result = await installFromMarket({
        pluginId: plugin.id,
        downloadUrl: plugin.download_url,
        repo: plugin.repo,
        downloadType: plugin.download_type,
        releaseAsset: plugin.release_asset,
        branch: plugin.branch,
      });
      await options.loadPlugins();

      if (result?.untrusted_url) {
        options.showFeedback("warning", i18n.t("market.untrusted_download_warning"));
      } else {
        options.showFeedback("success", i18n.t("market.install_success"));
      }

      pluginLogger.info("Market", "插件市场安装完成", {
        pluginId: plugin.id,
        untrustedUrl: Boolean(result?.untrusted_url),
      });
    } catch (err) {
      const errorMessage = normalizeErrorMessage(err);
      const hint = options.marketErrorHint.value || undefined;
      const extraHint = hint ? `\n${hint}` : "";
      options.showFeedback(
        "error",
        `${i18n.t("market.install_failed")}: ${errorMessage}${extraHint}`,
        0,
      );
      pluginLogger.error("Market", `插件市场安装失败: ${plugin.id}`, normalizeAppError(err));
    } finally {
      options.installing.value = null;
    }
  }

  return {
    loadMarket,
    showDetail,
    closeDetail,
    handleInstall,
  };
}
