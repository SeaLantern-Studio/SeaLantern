import { computed, onMounted, shallowRef } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import {
  getLocalizedPluginDescription,
  getLocalizedPluginName,
  type PluginEnableBlockReason,
  type PluginEnableGrantScope,
  type PluginInfo,
  type PluginState,
} from "@type/plugin";

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

function isPluginEnabled(plugin: PluginInfo): boolean {
  return plugin.state === "enabled";
}

function getMissingRequiredDependencies(plugin: PluginInfo) {
  return (plugin.missing_dependencies ?? []).filter((dependency) => dependency.required);
}

function getMissingOptionalDependencies(plugin: PluginInfo) {
  return (plugin.missing_dependencies ?? []).filter((dependency) => !dependency.required);
}

function buildClassicPluginDetailPath(pluginId: string): string {
  return `/plugin/${encodeURIComponent(pluginId)}`;
}

function navigateToClassicPath(path: string): void {
  window.location.assign(path);
}

function getPluginName(plugin: PluginInfo): string {
  return getLocalizedPluginName(plugin.manifest, i18n.getLocale());
}

function getPluginDescription(plugin: PluginInfo): string {
  return getLocalizedPluginDescription(plugin.manifest, i18n.getLocale());
}

function getPluginAuthor(plugin: PluginInfo): string | null {
  return plugin.manifest.author?.name ?? null;
}

function getPluginMeta(plugin: PluginInfo): string[] {
  const parts = [plugin.runtime.toUpperCase(), plugin.source];
  if (plugin.distribution_class && plugin.distribution_class !== "unknown") {
    parts.push(plugin.distribution_class);
  }
  return parts;
}

async function openRepository(url: string): Promise<void> {
  await openUrl(url);
}

function getPluginNameLabelFallback(): string {
  return i18n.t("plugins.title");
}

export function usePluginsPage() {
  const pluginStore = usePluginStore();

  const bootstrapping = shallowRef(true);
  const refreshing = shallowRef(false);
  const checkingUpdates = shallowRef(false);
  const loadedOnce = shallowRef(false);

  const permissionDialogOpen = shallowRef(false);
  const pendingPermissionPluginId = shallowRef<string | null>(null);
  const pendingPermissionPluginName = shallowRef("");
  const pendingPermissionList = shallowRef<string[]>([]);
  const pendingPermissionGrantScope = shallowRef<PluginEnableGrantScope>("version");
  const pendingPermissionBlockReason = shallowRef<PluginEnableBlockReason | null>(null);

  async function loadPage(manual = false): Promise<void> {
    if (manual) {
      refreshing.value = true;
    } else if (!loadedOnce.value) {
      bootstrapping.value = true;
    }

    try {
      if (manual) {
        await pluginStore.refreshPlugins();
      } else {
        await pluginStore.loadPlugins();
      }

      loadedOnce.value = true;

      await Promise.allSettled([pluginStore.loadNavItems(), pluginStore.checkAllUpdates()]);
    } finally {
      bootstrapping.value = false;
      refreshing.value = false;
    }
  }

  async function refreshPlugins(): Promise<void> {
    await loadPage(true);
  }

  async function checkAllUpdates(): Promise<void> {
    checkingUpdates.value = true;
    try {
      await pluginStore.checkAllUpdates();
    } finally {
      checkingUpdates.value = false;
    }
  }

  function clearError(): void {
    pluginStore.error = null;
  }

  function hasUpdate(plugin: PluginInfo): boolean {
    return Boolean(pluginStore.updates[plugin.manifest.id]);
  }

  function getUpdateSummary(plugin: PluginInfo): string | null {
    const update = pluginStore.updates[plugin.manifest.id];
    if (!update) {
      return null;
    }

    return i18n.t("plugins.next.update_to", { version: update.latest_version });
  }

  function canOpenClassicDetails(plugin: PluginInfo): boolean {
    const pluginId = plugin.manifest.id;
    return (
      pluginStore.navItems.some((item) => item.plugin_id === pluginId) ||
      Boolean(plugin.manifest.ui?.pages?.length) ||
      plugin.manifest.sidebar?.mode === "self" ||
      plugin.manifest.sidebar?.mode === "category"
    );
  }

  function openClassicDetails(pluginId: string): void {
    navigateToClassicPath(buildClassicPluginDetailPath(pluginId));
  }

  function openClassicMarket(): void {
    navigateToClassicPath("/market");
  }

  function closePermissionDialog(): void {
    permissionDialogOpen.value = false;
    pendingPermissionPluginId.value = null;
    pendingPermissionPluginName.value = "";
    pendingPermissionList.value = [];
    pendingPermissionGrantScope.value = "version";
    pendingPermissionBlockReason.value = null;
  }

  async function togglePlugin(plugin: PluginInfo, nextEnabled: boolean): Promise<void> {
    clearError();

    const result = await pluginStore.togglePlugin(plugin.manifest.id, nextEnabled);

    if (!result.success && result.error) {
      return;
    }

    if (!result.success && result.confirmationRequired) {
      const resultPlugin = result.plugin ?? plugin;
      pendingPermissionPluginId.value = plugin.manifest.id;
      pendingPermissionPluginName.value = getPluginName(resultPlugin);
      pendingPermissionList.value = resultPlugin.manifest.permissions ?? [];
      pendingPermissionGrantScope.value = result.grantScope ?? inferGrantScope(resultPlugin);
      pendingPermissionBlockReason.value = result.blockReason ?? null;
      permissionDialogOpen.value = true;
      return;
    }

    if (result.success) {
      await pluginStore.loadNavItems();
    }
  }

  async function confirmEnablePlugin(): Promise<void> {
    const pluginId = pendingPermissionPluginId.value;
    if (!pluginId) {
      return;
    }

    const result = await pluginStore.confirmEnablePlugin(pluginId, {
      grant_scope: pendingPermissionGrantScope.value,
    });

    if (result.success) {
      closePermissionDialog();
      await pluginStore.loadNavItems();
      return;
    }

    if (!result.confirmationRequired) {
      closePermissionDialog();
    }
  }

  const installedPlugins = computed(() => pluginStore.plugins);
  const totalCount = computed(() => installedPlugins.value.length);
  const enabledCount = computed(
    () => installedPlugins.value.filter((plugin) => isPluginEnabled(plugin)).length,
  );
  const updateCount = computed(
    () => installedPlugins.value.filter((plugin) => hasUpdate(plugin)).length,
  );
  const hasPlugins = computed(() => totalCount.value > 0);
  const errorMessage = computed(() => pluginStore.error);
  const isLoading = computed(() => bootstrapping.value || refreshing.value || pluginStore.loading);

  const permissionDialogTitle = computed(() => {
    if (pendingPermissionBlockReason.value === "revoked") {
      return i18n.t("plugins.next.permission_revoked_title");
    }

    return i18n.t("plugins.next.permission_confirm_title");
  });

  const permissionDialogMessage = computed(() => {
    if (pendingPermissionBlockReason.value === "revoked") {
      return i18n.t("plugins.next.permission_revoked_message", {
        name: pendingPermissionPluginName.value || getPluginNameLabelFallback(),
      });
    }

    return i18n.t("plugins.next.permission_confirm_message", {
      name: pendingPermissionPluginName.value || getPluginNameLabelFallback(),
    });
  });

  onMounted(() => {
    void loadPage(false);
  });

  return {
    pluginStore,
    installedPlugins,
    totalCount,
    enabledCount,
    updateCount,
    hasPlugins,
    isLoading,
    isBootstrapping: bootstrapping,
    isRefreshing: refreshing,
    checkingUpdates,
    errorMessage,
    permissionDialogOpen,
    pendingPermissionPluginName,
    pendingPermissionList,
    permissionDialogTitle,
    permissionDialogMessage,
    loadPage,
    refreshPlugins,
    checkAllUpdates,
    clearError,
    getPluginName,
    getPluginDescription,
    getPluginAuthor,
    getPluginMeta,
    getPluginStateLabel,
    getPluginStateTone,
    getMissingRequiredDependencies,
    getMissingOptionalDependencies,
    hasUpdate,
    getUpdateSummary,
    canOpenClassicDetails,
    openClassicDetails,
    openClassicMarket,
    openRepository,
    togglePlugin,
    closePermissionDialog,
    confirmEnablePlugin,
  };
}
