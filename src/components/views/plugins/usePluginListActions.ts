import { ref } from "vue";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import type { PluginInfo, PluginState } from "@type/plugin";
import { hasDangerousPermissions } from "@type/plugin";
import { RefreshCw, Trash2 } from "lucide-vue-next";

interface UsePluginListActionsOptions {
  plugins: () => PluginInfo[];
  clearError: () => void;
  togglePlugin: (
    pluginId: string,
    enable: boolean,
  ) => Promise<{ success: boolean; error?: string; disabledPlugins?: string[] }>;
  checkUpdate: (pluginId: string) => Promise<{
    latest_version: string;
    current_version: string;
  } | null>;
  checkAllUpdates: () => Promise<Array<{ plugin_id: string }>>;
  deletePlugin: (pluginId: string, deleteData?: boolean) => Promise<void>;
  deletePlugins: (pluginIds: string[], deleteData?: boolean) => Promise<void>;
  showAlert: (title: string, message: string) => void;
  getErrorMessage: (error: unknown) => string;
  openPermissionWarning: (pluginId: string, pluginName: string, permissions: string[]) => void;
  closePermissionWarning: () => void;
  pendingPermissionPluginId: () => string;
  prepareSingleDelete: (plugin: PluginInfo | undefined, pluginId: string) => void;
  clearSingleDeleteState: () => void;
  pendingDeletePluginId: () => string | null;
  selectedPluginIds: () => Set<string>;
  clearSelection: () => void;
  closeSingleDeleteDialog: () => void;
  closeBatchDeleteDialog: () => void;
  exitBatchMode: () => void;
}

export function usePluginListActions(options: UsePluginListActionsOptions) {
  const router = useRouter();
  const checkingUpdate = ref<string | null>(null);
  const checkingAllUpdates = ref(false);

  async function handleToggle(id: string, nextEnabled: boolean) {
    options.clearError();

    if (nextEnabled) {
      const plugin = options.plugins().find((item) => item.manifest.id === id);
      const permissions = plugin?.manifest.permissions || [];
      if (hasDangerousPermissions(permissions)) {
        options.openPermissionWarning(id, plugin?.manifest.name || id, permissions);
        return;
      }
    }

    await doTogglePlugin(id, nextEnabled);
  }

  async function confirmPermissionWarning() {
    const pluginId = options.pendingPermissionPluginId();
    options.closePermissionWarning();
    if (!pluginId) {
      return;
    }

    await doTogglePlugin(pluginId, true);
  }

  function cancelPermissionWarning() {
    options.closePermissionWarning();
  }

  async function doTogglePlugin(id: string, enable: boolean) {
    const result = await options.togglePlugin(id, enable);

    if (!result.success && result.error) {
      options.showAlert(i18n.t("plugins.enable_failed"), result.error);
      return;
    }

    if (result.disabledPlugins && result.disabledPlugins.length > 0) {
      const plugin = options.plugins().find((item) => item.manifest.id === id);
      const pluginName = plugin?.manifest.name || id;
      const disabledNames = result.disabledPlugins.map((depId) => {
        const dep = options.plugins().find((item) => item.manifest.id === depId);
        return dep?.manifest.name || depId;
      });
      options.showAlert(
        i18n.t("plugins.plugin_disabled"),
        i18n.t("plugins.plugin_disabled_desc", {
          name: pluginName,
          deps: disabledNames.join(", "),
        }),
      );
    }
  }

  async function handleDelete(pluginId: string) {
    const plugin = options.plugins().find((item) => item.manifest.id === pluginId);
    if (plugin?.state === "enabled") {
      options.showAlert(i18n.t("plugins.cannot_delete_enabled"), plugin.manifest.name);
      return;
    }

    options.prepareSingleDelete(plugin, pluginId);
  }

  async function executeSingleDelete(deleteData: boolean) {
    options.closeSingleDeleteDialog();
    const pluginId = options.pendingDeletePluginId();
    if (!pluginId) {
      return;
    }

    try {
      await options.deletePlugin(pluginId, deleteData);
    } catch (error) {
      options.showAlert(i18n.t("common.message_unknown_error"), options.getErrorMessage(error));
    } finally {
      options.clearSingleDeleteState();
    }
  }

  async function executeBatchDelete(deleteData: boolean) {
    options.closeBatchDeleteDialog();
    const ids = Array.from(options.selectedPluginIds());

    const enabledNames = ids
      .map((id) => options.plugins().find((item) => item.manifest.id === id))
      .filter((plugin) => plugin?.state === "enabled")
      .map((plugin) => plugin!.manifest.name);
    if (enabledNames.length > 0) {
      options.showAlert(i18n.t("plugins.cannot_delete_enabled"), enabledNames.join(", "));
      return;
    }

    try {
      await options.deletePlugins(ids, deleteData);
      options.clearSelection();
      options.exitBatchMode();
    } catch (error) {
      options.showAlert(i18n.t("common.message_unknown_error"), options.getErrorMessage(error));
    }
  }

  async function handleCheckUpdate(pluginId: string) {
    checkingUpdate.value = pluginId;
    try {
      const update = await options.checkUpdate(pluginId);
      if (update) {
        options.showAlert(
          i18n.t("plugins.new_version_found"),
          `${i18n.t("plugins.latest_version")}: ${update.latest_version}\n${i18n.t("plugins.current_version")}: ${update.current_version}`,
        );
      } else {
        options.showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.already_latest"));
      }
    } finally {
      checkingUpdate.value = null;
    }
  }

  async function handleCheckAllUpdates() {
    checkingAllUpdates.value = true;
    try {
      const updates = await options.checkAllUpdates();
      if (updates.length > 0) {
        options.showAlert(
          i18n.t("plugins.check_update"),
          i18n.t("plugins.updates_available", { count: updates.length }),
        );
      } else {
        options.showAlert(i18n.t("plugins.check_update"), i18n.t("plugins.all_plugins_latest"));
      }
    } finally {
      checkingAllUpdates.value = false;
    }
  }

  function getStatusColor(state: PluginState): string {
    if (typeof state === "object" && "error" in state) {
      return "var(--sl-error)";
    }
    switch (state) {
      case "enabled":
        return "var(--sl-success)";
      case "disabled":
        return "var(--sl-text-tertiary)";
      case "loaded":
        return "var(--sl-info)";
      default:
        return "var(--sl-text-secondary)";
    }
  }

  function getStatusLabel(state: PluginState): string {
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

  function isPluginEnabled(state: PluginState): boolean {
    return state === "enabled";
  }

  function hasSettings(plugin: PluginInfo): boolean {
    return !!(plugin.manifest.settings && plugin.manifest.settings.length > 0);
  }

  function getPluginMenuItems(pluginId: string) {
    return [
      {
        id: "check_update",
        label: i18n.t("plugins.menu.check_update"),
        icon: RefreshCw,
        disabled: checkingUpdate.value === pluginId,
      },
      { id: "divider", label: "", divider: true },
      {
        id: "delete",
        label: i18n.t("plugins.menu.delete"),
        icon: Trash2,
        danger: true,
      },
    ];
  }

  async function handleMenuSelect(item: { id: string | number }, pluginId: string) {
    switch (item.id) {
      case "check_update":
        await handleCheckUpdate(pluginId);
        break;
      case "delete":
        await handleDelete(pluginId);
        break;
    }
  }

  function goToMarket() {
    router.push("/plugins?tab=market");
  }

  return {
    checkingUpdate,
    checkingAllUpdates,
    handleToggle,
    confirmPermissionWarning,
    cancelPermissionWarning,
    handleDelete,
    executeSingleDelete,
    executeBatchDelete,
    handleCheckUpdate,
    handleCheckAllUpdates,
    getStatusColor,
    getStatusLabel,
    isPluginEnabled,
    hasSettings,
    getPluginMenuItems,
    handleMenuSelect,
    goToMarket,
  };
}
