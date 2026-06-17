import { computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { i18n } from "@language";
import type { MissingDependency, PluginInfo } from "@type/plugin";
import { getLocalizedPluginDescription, getLocalizedPluginName } from "@type/plugin";

function getPluginPermissionLabel(permission: string): string {
  const key = `plugins.permission.${permission}`;
  return i18n.t(key) !== key ? i18n.t(key) : permission;
}

function getPluginPermissionDesc(permission: string): string {
  const key = `plugins.permission.${permission}_desc`;
  return i18n.t(key) !== key ? i18n.t(key) : "";
}

function openPluginRepository(url: string) {
  void openUrl(url);
}

function getDisplayPluginName(plugin: PluginInfo): string {
  return getLocalizedPluginName(plugin.manifest, i18n.getLocale());
}

function getDisplayPluginDescription(plugin: PluginInfo): string {
  return getLocalizedPluginDescription(plugin.manifest, i18n.getLocale());
}

interface UsePluginsViewPageOptions {
  plugins: () => PluginInfo[];
  searchQuery: () => string;
  refreshPlugins: () => Promise<void> | void;
  getMissingRequiredDependencies: (plugin: PluginInfo) => MissingDependency[];
  getMissingOptionalDependencies: (plugin: PluginInfo) => MissingDependency[];
  saveSettings: () => Promise<string | null>;
  showAlert: (title: string, message?: string) => void;
  goToMarket: () => void;
  settingsForm: Record<string, string | number | boolean>;
  setInstalledPluginName: (name: string) => void;
  setMissingDependencies: (dependencies: MissingDependency[]) => void;
  setShowDependencyModal: (show: boolean) => void;
}

export function usePluginsViewPage(options: UsePluginsViewPageOptions) {
  const filteredPlugins = computed(() => {
    const query = options.searchQuery().trim().toLowerCase();
    if (!query) {
      return options.plugins();
    }

    return options.plugins().filter((plugin) => {
      const id = plugin.manifest.id.toLowerCase();
      const name = getLocalizedPluginName(plugin.manifest, i18n.getLocale()).toLowerCase();
      const state = (typeof plugin.state === "string" ? plugin.state : "error").toLowerCase();
      return id.includes(query) || name.includes(query) || state.includes(query);
    });
  });

  function handleRefresh() {
    void options.refreshPlugins();
  }

  function updateSettingsField(key: string, value: string | number | boolean) {
    options.settingsForm[key] = value;
  }

  function showMissingDependenciesModal(plugin: PluginInfo) {
    options.setInstalledPluginName(plugin.manifest.name);
    const required = options.getMissingRequiredDependencies(plugin);
    const optional = options.getMissingOptionalDependencies(plugin);
    options.setMissingDependencies([...required, ...optional]);
    options.setShowDependencyModal(true);
  }

  async function handleSaveSettings() {
    const errorMessage = await options.saveSettings();
    if (errorMessage) {
      options.showAlert(i18n.t("common.message_unknown_error"), errorMessage);
    }
  }

  function handleGoToMarket() {
    options.setShowDependencyModal(false);
    options.goToMarket();
  }

  return {
    filteredPlugins,
    handleRefresh,
    getPermissionLabel: getPluginPermissionLabel,
    getPermissionDesc: getPluginPermissionDesc,
    updateSettingsField,
    showMissingDependenciesModal,
    openRepository: openPluginRepository,
    getPluginName: getDisplayPluginName,
    getPluginDescription: getDisplayPluginDescription,
    handleSaveSettings,
    handleGoToMarket,
  };
}
