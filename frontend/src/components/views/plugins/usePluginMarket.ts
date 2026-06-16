import { onMounted, ref } from "vue";
import { type MarketPluginInfo } from "@api/plugin";
import {
  getMarketPermissionLevel,
  MARKET_BASE_URL,
  resolveMarketValue,
  type MarketFeedback,
  type MarketPlugin,
} from "@components/views/plugins/pluginMarketShared";
import { usePluginMarketActions } from "@components/views/plugins/usePluginMarketActions";
import { usePluginMarketFeedback } from "@components/views/plugins/usePluginMarketFeedback";
import { usePluginMarketSource } from "@components/views/plugins/usePluginMarketSource";
import { usePluginMarketViewState } from "@components/views/plugins/usePluginMarketViewState";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";

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

  const { showFeedback, clearFeedback } = usePluginMarketFeedback({
    installFeedback,
  });

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

  const { loadMarket, showDetail, closeDetail, handleInstall } = usePluginMarketActions({
    loading,
    error,
    marketPlugins,
    categories,
    installing,
    selectedPlugin,
    detailLoading,
    pluginDetail,
    marketErrorHint,
    getMarketRequestUrl,
    getSourceUrl: () => activeMarketUrl.value.trim().replace(/\/$/, "") || MARKET_BASE_URL,
    loadPlugins: pluginStore.loadPlugins,
    isInstalled,
    showFeedback,
  });

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
