import type { MarketPluginInfo } from "@api/plugin";
import type { MarketPlugin } from "./pluginMarketShared";
import {
  installPluginFromMarketEntry,
  loadPluginMarketCatalog,
  loadPluginMarketDetail,
} from "./pluginMarketActionsShared";
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

export function usePluginMarketActions(options: UsePluginMarketActionsOptions) {
  async function loadMarket() {
    options.loading.value = true;
    options.error.value = null;

    try {
      const data = await loadPluginMarketCatalog({
        requestUrl: options.getMarketRequestUrl(),
        sourceUrl: options.getSourceUrl(),
      });
      options.marketPlugins.value = data.plugins;
      options.categories.value = data.categories;
    } catch (err) {
      options.error.value = err instanceof Error ? err.message : String(err);
    } finally {
      options.loading.value = false;
    }
  }

  async function showDetail(plugin: MarketPlugin) {
    options.selectedPlugin.value = plugin;
    options.detailLoading.value = true;

    try {
      options.pluginDetail.value = await loadPluginMarketDetail({
        plugin,
        requestUrl: options.getMarketRequestUrl(),
        sourceUrl: options.getSourceUrl(),
      });
    } catch (err) {
      options.pluginDetail.value = null;
      options.error.value = err instanceof Error ? err.message : String(err);
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
      await installPluginFromMarketEntry({
        plugin,
        loadPlugins: options.loadPlugins,
        showFeedback: options.showFeedback,
        marketErrorHint: options.marketErrorHint.value,
      });
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
