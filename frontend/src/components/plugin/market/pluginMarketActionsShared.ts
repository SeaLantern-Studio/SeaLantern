import {
  fetchMarketCategories,
  fetchMarketPluginDetail,
  fetchMarketPlugins,
  installFromMarket,
  type MarketPluginInfo,
} from "@api/plugin";
import { formatPluginInstallIssue } from "@components/plugin/installer/pluginInstallErrorMessage";
import { MARKET_BASE_URL, type MarketPlugin, type MarketFeedbackType } from "./pluginMarketShared";
import { i18n } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";

export interface PluginMarketCatalogData {
  plugins: MarketPlugin[];
  categories: Record<string, Record<string, string> | string>;
}

export interface LoadPluginMarketCatalogOptions {
  requestUrl?: string;
  sourceUrl?: string;
}

export interface LoadPluginMarketDetailOptions {
  plugin: MarketPlugin;
  requestUrl?: string;
  sourceUrl?: string;
}

export interface InstallPluginFromMarketOptions {
  plugin: MarketPlugin;
  loadPlugins: () => Promise<void>;
  showFeedback: (type: MarketFeedbackType, message: string, duration?: number) => void;
  marketErrorHint?: string;
}

function normalizeErrorMessage(err: unknown): string {
  const normalized = normalizeAppError(err);
  const issueMessage = formatPluginInstallIssue({
    code: normalized.code,
    args: normalized.args,
  });
  return issueMessage || normalized.message;
}

export async function loadPluginMarketCatalog(
  options: LoadPluginMarketCatalogOptions,
): Promise<PluginMarketCatalogData> {
  const sourceUrl = options.sourceUrl || MARKET_BASE_URL;
  const [plugins, categories] = await Promise.all([
    fetchMarketPlugins(options.requestUrl),
    fetchMarketCategories(options.requestUrl).catch(() => ({})),
  ]);

  pluginLogger.info("Market", "插件市场列表已更新", {
    source: sourceUrl,
    pluginCount: plugins.length,
    categoryCount: Object.keys(categories).length,
  });

  return {
    plugins,
    categories,
  };
}

export async function loadPluginMarketDetail(
  options: LoadPluginMarketDetailOptions,
): Promise<MarketPluginInfo> {
  const sourceUrl = options.sourceUrl || MARKET_BASE_URL;
  const detail = await fetchMarketPluginDetail(
    options.plugin._path || `plugins/${options.plugin.id}.json`,
    options.requestUrl,
  );

  pluginLogger.info("Market", "插件详情已读取", {
    pluginId: options.plugin.id,
    source: sourceUrl,
  });

  return detail;
}

export async function installPluginFromMarketEntry(
  options: InstallPluginFromMarketOptions,
): Promise<void> {
  try {
    const result = await installFromMarket({
      pluginId: options.plugin.id,
      downloadUrl: options.plugin.download_url,
      repo: options.plugin.repo,
      downloadType: options.plugin.download_type,
      releaseAsset: options.plugin.release_asset,
      branch: options.plugin.branch,
      version: options.plugin.version,
    });

    await options.loadPlugins();

    const noticeMessages = (result?.install_notices || [])
      .map((notice) => formatPluginInstallIssue(notice))
      .filter((message): message is string => Boolean(message));

    if (result?.untrusted_url) {
      options.showFeedback("warning", i18n.t("market.untrusted_download_warning"));
    } else if (noticeMessages.length > 0) {
      options.showFeedback("warning", noticeMessages.join("\n"), 0);
    } else {
      options.showFeedback("success", i18n.t("market.install_success"));
    }

    pluginLogger.info("Market", "插件市场安装完成", {
      pluginId: options.plugin.id,
      untrustedUrl: Boolean(result?.untrusted_url),
    });
  } catch (err) {
    const errorMessage = normalizeErrorMessage(err);
    const extraHint = options.marketErrorHint ? `\n${options.marketErrorHint}` : "";
    options.showFeedback(
      "error",
      `${i18n.t("market.install_failed")}: ${errorMessage}${extraHint}`,
      0,
    );
    pluginLogger.error("Market", `插件市场安装失败: ${options.plugin.id}`, normalizeAppError(err));
  }
}
