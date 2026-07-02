import { onMounted, shallowRef, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { usePluginStore } from "@stores/pluginStore";
import { usePluginMarketFeedback } from "@components/plugin/market/usePluginMarketFeedback";
import { usePluginMarketSource } from "@components/plugin/market/usePluginMarketSource";
import { usePluginMarketViewState } from "@components/plugin/market/usePluginMarketViewState";
import {
  installPluginFromMarketEntry,
  loadPluginMarketCatalog,
} from "@components/plugin/market/pluginMarketActionsShared";
import {
  getMarketPermissionLevel,
  resolveMarketValue,
  type MarketFeedback,
  type MarketPlugin,
} from "@components/plugin/market/pluginMarketShared";
import { i18n } from "@language";
import { NEXT_PLUGIN_DETAIL_ROUTE_NAME } from "@src/router/pageMeta";

function getMarketPermissionLabel(perm: string): string {
  return i18n.t(`plugins.permission.${perm}`) !== `plugins.permission.${perm}`
    ? i18n.t(`plugins.permission.${perm}`)
    : perm;
}

function getMarketPermissionDesc(perm: string): string {
  return i18n.t(`plugins.permission.${perm}_desc`) !== `plugins.permission.${perm}_desc`
    ? i18n.t(`plugins.permission.${perm}_desc`)
    : "";
}

export function usePluginMarketPage() {
  const route = useRoute();
  const router = useRouter();
  const pluginStore = usePluginStore();

  const loading = shallowRef(true);
  const error = shallowRef<string | null>(null);
  const installFeedback = shallowRef<MarketFeedback | null>(null);
  const marketPlugins = shallowRef<MarketPlugin[]>([]);
  const categories = shallowRef<Record<string, Record<string, string> | string>>({});
  const searchQuery = shallowRef("");
  const selectedTag = shallowRef<string | null>(null);
  const installing = shallowRef<string | null>(null);

  const { showFeedback, clearFeedback } = usePluginMarketFeedback({
    installFeedback,
  });

  async function loadMarket() {
    loading.value = true;
    error.value = null;

    try {
      const data = await loadPluginMarketCatalog({
        requestUrl: getMarketRequestUrl(),
        sourceUrl: activeMarketUrl.value,
      });
      marketPlugins.value = data.plugins;
      categories.value = data.categories;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  }

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

  async function installPlugin(plugin: MarketPlugin): Promise<void> {
    if (installing.value || isInstalled(plugin.id)) {
      return;
    }

    installing.value = plugin.id;
    try {
      await installPluginFromMarketEntry({
        plugin,
        loadPlugins: pluginStore.loadPlugins,
        showFeedback,
        marketErrorHint: marketErrorHint.value,
      });
    } finally {
      installing.value = null;
    }
  }

  async function openPluginDetail(plugin: MarketPlugin): Promise<void> {
    await router.push({
      name: NEXT_PLUGIN_DETAIL_ROUTE_NAME,
      params: { pluginId: plugin.id },
      query: {
        ...route.query,
        pluginsContext: "market",
      },
    });
  }

  watch(
    () => route.query.tag,
    (value) => {
      selectedTag.value = typeof value === "string" && value ? value : null;
    },
    { immediate: true },
  );

  watch(selectedTag, async (value) => {
    const currentTag = typeof route.query.tag === "string" ? route.query.tag : null;
    const nextTag = value || null;
    if (currentTag === nextTag) {
      return;
    }

    const nextQuery = { ...route.query };
    if (nextTag) {
      nextQuery.tag = nextTag;
    } else {
      delete nextQuery.tag;
    }

    await router.replace({ query: nextQuery });
  });

  onMounted(() => {
    if (!pluginStore.plugins.length && !pluginStore.loading) {
      void pluginStore.loadPlugins();
    }

    void loadMarket();
  });

  return {
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
    resolveI18n: resolveMarketValue,
    isInstalled,
    isInstalledAndEnabled,
    getInstallButtonText,
    getPermissionLevel: getMarketPermissionLevel,
    getPermissionLabel: getMarketPermissionLabel,
    getPermissionDesc: getMarketPermissionDesc,
    getCategoryLabel,
    getIconUrl,
    loadMarket,
    installPlugin,
    openPluginDetail,
  };
}
