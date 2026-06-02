import { computed, onMounted, ref } from "vue";
import {
  fetchMarketCategories,
  fetchMarketPluginDetail,
  fetchMarketPlugins,
  installFromMarket,
  type MarketPluginInfo,
} from "@api/plugin";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import { normalizeAppError } from "@utils/appError";

export type MarketPlugin = MarketPluginInfo & { _path?: string };
export type MarketFeedbackType = "success" | "warning" | "error";

export interface MarketFeedback {
  type: MarketFeedbackType;
  message: string;
}

interface ValidatedMarketSource {
  url: string;
  custom: boolean;
  host: string;
  protocol: string;
}

const MARKET_BASE_URL = "https://sealantern-studio.github.io/plugin-market";
const MARKET_URL_KEY = "sealantern_market_url";
const CRITICAL_PERMS = new Set(["execute_program", "plugin_folder_access"]);
const DANGEROUS_PERMS = new Set(["fs", "network", "server", "console"]);

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
  const showUrlEditor = ref(false);
  const customMarketUrl = ref(localStorage.getItem(MARKET_URL_KEY) || "");
  const urlInput = ref(customMarketUrl.value);
  const pendingMarketSource = ref<string | null>(null);
  const showCustomSourceConfirm = ref(false);

  const validatedMarketSource = computed<ValidatedMarketSource>(() => {
    const validated = validateMarketUrl(customMarketUrl.value);
    if (validated) {
      return validated;
    }

    const fallback = new URL(MARKET_BASE_URL);
    return {
      url: MARKET_BASE_URL,
      custom: false,
      host: fallback.host,
      protocol: fallback.protocol,
    };
  });

  const activeMarketUrl = computed(() => validatedMarketSource.value.url);
  const activeMarketHost = computed(() => validatedMarketSource.value.host);
  const isUsingCustomMarket = computed(() => validatedMarketSource.value.custom);
  const marketErrorHint = computed<string>(() => {
    if (!error.value) {
      return "";
    }
    return resolveMarketNetworkHint(error.value);
  });
  const filteredPlugins = computed(() => {
    let result = marketPlugins.value;
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        (plugin) =>
          resolveI18n(plugin.name).toLowerCase().includes(q) ||
          resolveI18n(plugin.description).toLowerCase().includes(q) ||
          plugin.author?.name?.toLowerCase().includes(q),
      );
    }
    if (selectedTag.value) {
      result = result.filter((plugin) => plugin.categories?.includes(selectedTag.value!));
    }
    return result;
  });
  const allTags = computed(() => {
    const tags = new Set<string>();
    marketPlugins.value.forEach((plugin) => plugin.categories?.forEach((tag) => tags.add(tag)));
    return Array.from(tags);
  });
  const tagTabs = computed(() => [
    { key: null, label: i18n.t("config.categories.all") },
    ...allTags.value.map((tag) => ({ key: tag, label: getCategoryLabel(tag) })),
  ]);

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

  function resolveI18n(value: Record<string, string> | string | undefined): string {
    if (!value) {
      return "";
    }
    if (typeof value === "string") {
      return value;
    }
    const locale = i18n.getLocale();
    const localeKey = locale.startsWith("zh") ? "zh-CN" : "en-US";
    return value[localeKey] || value["zh-CN"] || Object.values(value)[0] || "";
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

  function getPermissionLevel(perm: string): "critical" | "dangerous" | "normal" {
    if (CRITICAL_PERMS.has(perm)) {
      return "critical";
    }
    if (DANGEROUS_PERMS.has(perm)) {
      return "dangerous";
    }
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
    const locale = i18n.getLocale();
    const localeKey = locale.startsWith("zh") ? "zh-CN" : "en-US";
    const category = categories.value[key];
    if (!category) {
      return key;
    }
    if (typeof category === "string") {
      return category;
    }
    return category[localeKey] || category["zh-CN"] || key;
  }

  function getIconUrl(plugin: MarketPlugin): string | null {
    if (!plugin.icon_url || !plugin._path) {
      return null;
    }
    const dir = plugin._path.replace(/\/[^/]+$/, "");
    return `${activeMarketUrl.value.trim().replace(/\/$/, "")}/${dir}/${plugin.icon_url}`;
  }

  function validateMarketUrl(input: string): ValidatedMarketSource | null {
    const trimmed = input.trim();
    if (!trimmed) {
      return null;
    }

    try {
      const parsed = new URL(trimmed);
      const isHttps = parsed.protocol === "https:";
      const isLocalHttp =
        parsed.protocol === "http:" && ["localhost", "127.0.0.1", "::1"].includes(parsed.hostname);
      if (!isHttps && !isLocalHttp) {
        return null;
      }

      parsed.hash = "";
      return {
        url: parsed.toString().replace(/\/$/, ""),
        custom: parsed.toString().replace(/\/$/, "") !== MARKET_BASE_URL,
        host: parsed.host,
        protocol: parsed.protocol,
      };
    } catch {
      return null;
    }
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
      const [plugins, categoryMap] = await Promise.all([
        fetchMarketPlugins(url === MARKET_BASE_URL ? undefined : url),
        fetchMarketCategories(url === MARKET_BASE_URL ? undefined : url).catch(() => ({})),
      ]);
      marketPlugins.value = plugins;
      categories.value = categoryMap;
      pluginLogger.info("Market", "插件市场列表已更新", {
        source: url || MARKET_BASE_URL,
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
      const url = activeMarketUrl.value.trim().replace(/\/$/, "");
      pluginDetail.value = await fetchMarketPluginDetail(
        plugin._path || `plugins/${plugin.id}.json`,
        url === MARKET_BASE_URL ? undefined : url,
      );
      pluginLogger.info("Market", "插件详情已读取", {
        pluginId: plugin.id,
        source: url || MARKET_BASE_URL,
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
      const hint = resolveMarketNetworkHint(errorMessage);
      const extraHint = hint ? `\n${hint}` : "";
      showFeedback("error", `${i18n.t("market.install_failed")}: ${errorMessage}${extraHint}`, 0);
      pluginLogger.error("Market", `插件市场安装失败: ${plugin.id}`, normalizeAppError(err));
    } finally {
      installing.value = null;
    }
  }

  function toggleUrlEditor() {
    showUrlEditor.value = !showUrlEditor.value;
  }

  function saveMarketUrl() {
    const url = urlInput.value.trim();
    if (!url) {
      resetMarketUrl();
      return;
    }

    const validated = validateMarketUrl(url);
    if (!validated) {
      showFeedback("error", i18n.t("market.source_invalid"));
      return;
    }

    if (validated.custom) {
      pendingMarketSource.value = validated.url;
      showCustomSourceConfirm.value = true;
      return;
    }

    applyMarketUrl(validated.url);
  }

  function applyMarketUrl(url: string) {
    const normalized = url.trim();
    customMarketUrl.value = normalized === MARKET_BASE_URL ? "" : normalized;
    urlInput.value = customMarketUrl.value;
    if (customMarketUrl.value) {
      localStorage.setItem(MARKET_URL_KEY, customMarketUrl.value);
    } else {
      localStorage.removeItem(MARKET_URL_KEY);
    }
    showUrlEditor.value = false;
    showCustomSourceConfirm.value = false;
    pendingMarketSource.value = null;
    pluginLogger.info("Market", "插件市场来源已更新", {
      source: customMarketUrl.value || MARKET_BASE_URL,
      custom: Boolean(customMarketUrl.value),
    });
    void loadMarket();
  }

  function confirmCustomMarketUrl() {
    if (!pendingMarketSource.value) {
      showCustomSourceConfirm.value = false;
      return;
    }

    applyMarketUrl(pendingMarketSource.value);
  }

  function cancelCustomMarketUrl() {
    pendingMarketSource.value = null;
    showCustomSourceConfirm.value = false;
  }

  function resetMarketUrl() {
    urlInput.value = "";
    customMarketUrl.value = "";
    pendingMarketSource.value = null;
    localStorage.removeItem(MARKET_URL_KEY);
    showUrlEditor.value = false;
    showCustomSourceConfirm.value = false;
    pluginLogger.info("Market", "插件市场来源已恢复默认", {
      source: MARKET_BASE_URL,
    });
    void loadMarket();
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
  };
}
