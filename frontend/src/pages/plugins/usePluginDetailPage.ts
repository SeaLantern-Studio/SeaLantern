import { computed, onMounted, shallowRef, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { MarketPluginInfo } from "@api/plugin";
import {
  installPluginFromMarketEntry,
  loadPluginMarketCatalog,
  loadPluginMarketDetail,
} from "@components/plugin/market/pluginMarketActionsShared";
import { usePluginMarketFeedback } from "@components/plugin/market/usePluginMarketFeedback";
import {
  MARKET_BASE_URL,
  MARKET_URL_KEY,
  type MarketFeedback,
  type MarketPlugin,
  validateMarketUrl,
} from "@components/plugin/market/pluginMarketShared";
import { i18n } from "@language";
import { NEXT_PLUGIN_CATEGORY_ROUTE_NAME } from "@src/router/pageMeta";
import { usePluginStore } from "@stores/pluginStore";
import {
  getLocalizedPluginDescription,
  getLocalizedPluginName,
  type PluginEnableBlockReason,
  type PluginEnableGrantScope,
  type PluginInfo,
  type PluginState,
} from "@type/plugin";
import { usePluginPageSettings } from "./usePluginPageSettings";

const loadedPluginDetailIds = new Set<string>();

interface PluginDetailMarketState {
  pluginId: string;
  plugin: MarketPlugin | null;
  detail: MarketPluginInfo | null;
}

interface PluginDetailErrorState {
  pluginId: string;
  message: string | null;
}

function getPluginStateLabel(state: PluginState): string {
  if (typeof state === "object" && "error" in state) {
    return i18n.t("plugins.status.error");
  }

  switch (state) {
    case "enabled":
      return i18n.t("plugins.status.enabled");
    case "disabled":
      return i18n.t("plugins.status.disabled");
    case "loaded":
      return i18n.t("plugins.status.loaded");
    default:
      return String(state);
  }
}

function getPluginStateTone(state: PluginState): "success" | "warning" | "neutral" | "error" {
  if (typeof state === "object" && "error" in state) {
    return "error";
  }

  switch (state) {
    case "enabled":
      return "success";
    case "loaded":
      return "warning";
    case "disabled":
    default:
      return "neutral";
  }
}

function inferGrantScope(plugin: PluginInfo): PluginEnableGrantScope {
  switch (plugin.trust_level_display) {
    case "trusted":
      return "hash";
    case "unreviewed":
      return "version";
    case "standard_sandbox":
      return "version";
    case "builtin":
    default:
      return "version";
  }
}

function resolveMarketSourceUrl(): string {
  const stored = localStorage.getItem(MARKET_URL_KEY) || "";
  const validated = validateMarketUrl(stored);
  return validated?.url || MARKET_BASE_URL;
}

function buildMarketIconUrl(plugin: MarketPlugin | null, marketUrl: string): string | null {
  if (!plugin?.icon_url || !plugin._path) {
    return null;
  }

  const dir = plugin._path.replace(/\/[^/]+$/, "");
  return `${marketUrl.trim().replace(/\/$/, "")}/${dir}/${plugin.icon_url}`;
}

function resolveLocalizedMarketField(
  value: Record<string, string> | string | undefined,
  locale: string,
): string {
  if (!value) {
    return "";
  }

  if (typeof value === "string") {
    return value;
  }

  const localeKey = locale.startsWith("zh") ? "zh-CN" : "en-US";
  return value[localeKey] || value["zh-CN"] || Object.values(value)[0] || "";
}

export function usePluginDetailPage() {
  const route = useRoute();
  const router = useRouter();
  const pluginStore = usePluginStore();

  let loadRequestSequence = 0;

  const pluginId = computed(() => {
    return typeof route.params.pluginId === "string" ? route.params.pluginId : "";
  });

  const bootstrapping = shallowRef(!loadedPluginDetailIds.has(pluginId.value));
  const loadingPluginId = shallowRef<string | null>(null);
  const installFeedback = shallowRef<MarketFeedback | null>(null);
  const marketState = shallowRef<PluginDetailMarketState>({
    pluginId: "",
    plugin: null,
    detail: null,
  });
  const errorState = shallowRef<PluginDetailErrorState>({
    pluginId: "",
    message: null,
  });
  const installing = shallowRef(false);

  const permissionDialogOpen = shallowRef(false);
  const pendingPermissionPluginId = shallowRef<string | null>(null);
  const pendingPermissionPluginName = shallowRef("");
  const pendingPermissionList = shallowRef<string[]>([]);
  const pendingPermissionGrantScope = shallowRef<PluginEnableGrantScope>("version");
  const pendingPermissionBlockReason = shallowRef<PluginEnableBlockReason | null>(null);

  const { showFeedback, clearFeedback } = usePluginMarketFeedback({
    installFeedback,
  });

  const pageSettings = usePluginPageSettings({
    pluginId: () => pluginId.value,
  });

  const marketLoading = computed(() => loadingPluginId.value === pluginId.value);
  const errorMessage = computed(() => {
    return errorState.value.pluginId === pluginId.value ? errorState.value.message : null;
  });
  const marketPlugin = computed(() => {
    return marketState.value.pluginId === pluginId.value ? marketState.value.plugin : null;
  });
  const marketDetail = computed(() => {
    return marketState.value.pluginId === pluginId.value ? marketState.value.detail : null;
  });
  const installedPlugin = computed<PluginInfo | null>(() => {
    return pluginStore.plugins.find((plugin) => plugin.manifest.id === pluginId.value) ?? null;
  });
  const settingsReady = computed(() => pageSettings.plugin.value?.manifest.id === pluginId.value);

  const hasUpdate = computed(() => Boolean(pluginStore.updates[pluginId.value]));
  const marketSourceUrl = computed(() => resolveMarketSourceUrl());
  const iconUrl = computed(() => {
    if (installedPlugin.value) {
      return pluginStore.icons[installedPlugin.value.manifest.id] || null;
    }

    return buildMarketIconUrl(marketPlugin.value, marketSourceUrl.value);
  });

  const displayName = computed(() => {
    if (installedPlugin.value) {
      return getLocalizedPluginName(installedPlugin.value.manifest, i18n.getLocale());
    }

    return resolveLocalizedMarketField(
      marketDetail.value?.name || marketPlugin.value?.name,
      i18n.getLocale(),
    ) || pluginId.value;
  });

  const displayDescription = computed(() => {
    if (installedPlugin.value) {
      return getLocalizedPluginDescription(installedPlugin.value.manifest, i18n.getLocale());
    }

    return resolveLocalizedMarketField(
      marketDetail.value?.description || marketPlugin.value?.description,
      i18n.getLocale(),
    );
  });

  const authorName = computed(() => {
    return (
      installedPlugin.value?.manifest.author?.name ??
      marketDetail.value?.author?.name ??
      marketPlugin.value?.author?.name ??
      null
    );
  });

  const versionLabel = computed(() => {
    return (
      installedPlugin.value?.manifest.version ??
      marketDetail.value?.version ??
      marketPlugin.value?.version ??
      null
    );
  });

  const categoryTags = computed(() => {
    return marketDetail.value?.categories || marketPlugin.value?.categories || [];
  });

  const stateLabel = computed(() => {
    return installedPlugin.value ? getPluginStateLabel(installedPlugin.value.state) : null;
  });

  const stateTone = computed(() => {
    return installedPlugin.value ? getPluginStateTone(installedPlugin.value.state) : null;
  });

  const updateSummary = computed(() => {
    const update = pluginStore.updates[pluginId.value];
    if (!update) {
      return null;
    }

    return i18n.t("plugins.next.update_to", { version: update.latest_version });
  });

  const supportsSettingsOnDetail = computed(() => {
    return Boolean(
      settingsReady.value &&
        installedPlugin.value &&
        installedPlugin.value.manifest.sidebar?.mode !== "category",
    );
  });

  const canOpenCategoryPage = computed(() => {
    return installedPlugin.value?.manifest.sidebar?.mode === "category";
  });

  const showMarketInstallAction = computed(() => {
    if (!marketPlugin.value) {
      return false;
    }

    if (!installedPlugin.value) {
      return true;
    }

    return hasUpdate.value;
  });

  const marketActionLabel = computed(() => {
    if (installing.value) {
      return i18n.t("market.installing");
    }
    if (installedPlugin.value?.state === "enabled") {
      return i18n.t("market.running_need_disable");
    }
    if (hasUpdate.value && updateSummary.value) {
      return updateSummary.value;
    }
    return i18n.t("market.install");
  });

  const notFound = computed(() => {
    return !bootstrapping.value && !installedPlugin.value && !marketPlugin.value;
  });

  function isLatestRequest(requestId: number, targetPluginId: string): boolean {
    return requestId === loadRequestSequence && targetPluginId === pluginId.value;
  }

  function setErrorState(targetPluginId: string, message: string | null): void {
    errorState.value = {
      pluginId: targetPluginId,
      message,
    };
  }

  function resetMarketState(targetPluginId: string): void {
    marketState.value = {
      pluginId: targetPluginId,
      plugin: null,
      detail: null,
    };
  }

  function hasInstalledPlugin(targetPluginId: string): boolean {
    return pluginStore.plugins.some((plugin) => plugin.manifest.id === targetPluginId);
  }

  const permissionDialogTitle = computed(() => {
    if (pendingPermissionBlockReason.value === "revoked") {
      return i18n.t("plugins.next.permission_revoked_title");
    }

    return i18n.t("plugins.next.permission_confirm_title");
  });

  const permissionDialogMessage = computed(() => {
    if (pendingPermissionBlockReason.value === "revoked") {
      return i18n.t("plugins.next.permission_revoked_message", {
        name: pendingPermissionPluginName.value || displayName.value,
      });
    }

    return i18n.t("plugins.next.permission_confirm_message", {
      name: pendingPermissionPluginName.value || displayName.value,
    });
  });

  async function loadPage(targetPluginId = pluginId.value): Promise<void> {
    const requestId = ++loadRequestSequence;

    if (!loadedPluginDetailIds.has(targetPluginId)) {
      bootstrapping.value = true;
    }

    loadingPluginId.value = targetPluginId;
    setErrorState(targetPluginId, null);
    resetMarketState(targetPluginId);

    try {
      if (!pluginStore.plugins.length) {
        await pluginStore.loadPlugins();
      }

      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      await Promise.allSettled([pluginStore.loadNavItems(), pluginStore.checkAllUpdates()]);

      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      await pageSettings.loadPlugin();

      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      const requestUrl = marketSourceUrl.value === MARKET_BASE_URL ? undefined : marketSourceUrl.value;
      const catalog = await loadPluginMarketCatalog({
        requestUrl,
        sourceUrl: marketSourceUrl.value,
      });

      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      const nextMarketPlugin = catalog.plugins.find((plugin) => plugin.id === targetPluginId) ?? null;
      marketState.value = {
        pluginId: targetPluginId,
        plugin: nextMarketPlugin,
        detail: null,
      };

      if (nextMarketPlugin) {
        try {
          const nextMarketDetail = await loadPluginMarketDetail({
            plugin: nextMarketPlugin,
            requestUrl,
            sourceUrl: marketSourceUrl.value,
          });

          if (!isLatestRequest(requestId, targetPluginId)) {
            return;
          }

          marketState.value = {
            pluginId: targetPluginId,
            plugin: nextMarketPlugin,
            detail: nextMarketDetail,
          };
        } catch (err) {
          if (!isLatestRequest(requestId, targetPluginId)) {
            return;
          }

          marketState.value = {
            pluginId: targetPluginId,
            plugin: nextMarketPlugin,
            detail: null,
          };

          if (!hasInstalledPlugin(targetPluginId)) {
            setErrorState(targetPluginId, err instanceof Error ? err.message : String(err));
          }
        }
      }

      if (targetPluginId) {
        loadedPluginDetailIds.add(targetPluginId);
      }
    } catch (err) {
      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      if (!hasInstalledPlugin(targetPluginId)) {
        setErrorState(targetPluginId, err instanceof Error ? err.message : String(err));
      }
    } finally {
      if (!isLatestRequest(requestId, targetPluginId)) {
        return;
      }

      loadingPluginId.value = null;
      bootstrapping.value = false;
    }
  }

  async function installOrUpdatePlugin(): Promise<void> {
    if (!marketPlugin.value || installing.value) {
      return;
    }

    installing.value = true;
    try {
      await installPluginFromMarketEntry({
        plugin: marketPlugin.value,
        loadPlugins: pluginStore.loadPlugins,
        showFeedback,
      });
      await pluginStore.checkAllUpdates();
      await loadPage();
    } finally {
      installing.value = false;
    }
  }

  async function toggleInstalledPlugin(nextEnabled: boolean): Promise<void> {
    if (!installedPlugin.value) {
      return;
    }

    const plugin = installedPlugin.value;
    const result = await pluginStore.togglePlugin(plugin.manifest.id, nextEnabled);
    if (!result.success && result.confirmationRequired) {
      const resultPlugin = result.plugin ?? plugin;
      pendingPermissionPluginId.value = plugin.manifest.id;
      pendingPermissionPluginName.value = getLocalizedPluginName(resultPlugin.manifest, i18n.getLocale());
      pendingPermissionList.value = resultPlugin.manifest.permissions ?? [];
      pendingPermissionGrantScope.value = result.grantScope ?? inferGrantScope(resultPlugin);
      pendingPermissionBlockReason.value = result.blockReason ?? null;
      permissionDialogOpen.value = true;
      return;
    }

    if (result.success) {
      await pluginStore.loadNavItems();
      await pluginStore.checkAllUpdates();
    }
  }

  function closePermissionDialog(): void {
    permissionDialogOpen.value = false;
    pendingPermissionPluginId.value = null;
    pendingPermissionPluginName.value = "";
    pendingPermissionList.value = [];
    pendingPermissionGrantScope.value = "version";
    pendingPermissionBlockReason.value = null;
  }

  async function confirmEnablePlugin(): Promise<void> {
    const pendingPluginId = pendingPermissionPluginId.value;
    if (!pendingPluginId) {
      return;
    }

    const result = await pluginStore.confirmEnablePlugin(pendingPluginId, {
      grant_scope: pendingPermissionGrantScope.value,
    });

    if (result.success) {
      closePermissionDialog();
      await pluginStore.loadNavItems();
      await pluginStore.checkAllUpdates();
      return;
    }

    if (!result.confirmationRequired) {
      closePermissionDialog();
    }
  }

  async function openCategoryPage(): Promise<void> {
    if (!pluginId.value) {
      return;
    }

    await router.push({
      name: NEXT_PLUGIN_CATEGORY_ROUTE_NAME,
      params: { pluginId: pluginId.value },
      query: route.query,
    });
  }

  async function openRepository(): Promise<void> {
    const url =
      installedPlugin.value?.manifest.repository ??
      marketDetail.value?.author?.url ??
      marketPlugin.value?.repo;
    if (!url) {
      return;
    }

    await openUrl(url);
  }

  function updateMainSettingsField(key: string, value: string | number | boolean) {
    pageSettings.updateSettingsField(pageSettings.settingsForm, key, value);
  }

  function updateDependentField(pluginIdValue: string, key: string, value: string | number | boolean) {
    const form = pageSettings.dependentSettingsForms[pluginIdValue];
    if (!form) {
      return;
    }

    pageSettings.updateSettingsField(form, key, value);
  }

  watch(
    pluginId,
    (nextPluginId) => {
      if (!nextPluginId) {
        loadRequestSequence += 1;
        loadingPluginId.value = null;
        resetMarketState("");
        setErrorState("", i18n.t("plugins.not_found"));
        bootstrapping.value = false;
        return;
      }

      void loadPage(nextPluginId);
    },
    { immediate: true },
  );

  onMounted(() => {
    if (!pluginId.value) {
      bootstrapping.value = false;
      setErrorState("", i18n.t("plugins.not_found"));
    }
  });

  return {
    bootstrapping,
    marketLoading,
    errorMessage,
    installFeedback,
    installing,
    installedPlugin,
    marketPlugin,
    marketDetail,
    displayName,
    displayDescription,
    authorName,
    versionLabel,
    categoryTags,
    iconUrl,
    stateLabel,
    stateTone,
    updateSummary,
    supportsSettingsOnDetail,
    canOpenCategoryPage,
    showMarketInstallAction,
    marketActionLabel,
    notFound,
    permissionDialogOpen,
    pendingPermissionPluginName,
    pendingPermissionList,
    permissionDialogTitle,
    permissionDialogMessage,
    clearFeedback,
    loadPage,
    installOrUpdatePlugin,
    toggleInstalledPlugin,
    closePermissionDialog,
    confirmEnablePlugin,
    openCategoryPage,
    openRepository,
    pageSettings,
    updateMainSettingsField,
    updateDependentField,
    resolveLocalizedMarketField,
  };
}
