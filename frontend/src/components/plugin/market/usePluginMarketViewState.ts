import { computed, type Ref } from "vue";
import { i18n } from "@language";
import {
  MARKET_BASE_URL,
  resolveMarketNetworkHint,
  resolveMarketValue,
  type MarketPlugin,
} from "./pluginMarketShared";

interface UsePluginMarketViewStateOptions {
  error: Ref<string | null>;
  marketPlugins: Ref<MarketPlugin[]>;
  categories: Ref<Record<string, Record<string, string> | string>>;
  searchQuery: Ref<string>;
  selectedTag: Ref<string | null>;
  activeMarketUrl: Ref<string>;
}

export function usePluginMarketViewState(options: UsePluginMarketViewStateOptions) {
  const marketErrorHint = computed<string>(() => {
    if (!options.error.value) {
      return "";
    }

    return resolveMarketNetworkHint(options.error.value);
  });

  const filteredPlugins = computed(() => {
    let result = options.marketPlugins.value;
    if (options.searchQuery.value) {
      const query = options.searchQuery.value.toLowerCase();
      result = result.filter(
        (plugin) =>
          resolveMarketValue(plugin.name).toLowerCase().includes(query) ||
          resolveMarketValue(plugin.description).toLowerCase().includes(query) ||
          plugin.author?.name?.toLowerCase().includes(query),
      );
    }

    if (options.selectedTag.value) {
      result = result.filter((plugin) => plugin.categories?.includes(options.selectedTag.value!));
    }

    return result;
  });

  const allTags = computed(() => {
    const tags = new Set<string>();
    options.marketPlugins.value.forEach((plugin) =>
      plugin.categories?.forEach((tag) => tags.add(tag)),
    );
    return Array.from(tags);
  });

  function getCategoryLabel(key: string): string {
    const locale = i18n.getLocale();
    const localeKey = locale.startsWith("zh") ? "zh-CN" : "en-US";
    const category = options.categories.value[key];
    if (!category) {
      return key;
    }

    if (typeof category === "string") {
      return category;
    }

    return category[localeKey] || category["zh-CN"] || key;
  }

  const tagTabs = computed(() => [
    { key: null, label: i18n.t("config.categories.all") },
    ...allTags.value.map((tag) => ({ key: tag, label: getCategoryLabel(tag) })),
  ]);

  function getIconUrl(plugin: MarketPlugin): string | null {
    if (!plugin.icon_url || !plugin._path) {
      return null;
    }

    const dir = plugin._path.replace(/\/[^/]+$/, "");
    return `${options.activeMarketUrl.value.trim().replace(/\/$/, "")}/${dir}/${plugin.icon_url}`;
  }

  function getMarketRequestUrl(): string | undefined {
    const url = options.activeMarketUrl.value.trim().replace(/\/$/, "");
    return url === MARKET_BASE_URL ? undefined : url;
  }

  return {
    marketErrorHint,
    filteredPlugins,
    allTags,
    tagTabs,
    getCategoryLabel,
    getIconUrl,
    getMarketRequestUrl,
  };
}
