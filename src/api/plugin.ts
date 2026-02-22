import { tauriInvoke } from "@api/tauri";
import type {
  PluginInfo,
  PluginNavItem,
  PluginInstallResult,
  BatchInstallResult,
  PluginUpdateInfo,
} from "@type/plugin";

// 插件市场的信息
export interface MarketPluginInfo {
  id: string;
  name: Record<string, string> | string;
  description: Record<string, string> | string;
  repo?: string;
  version?: string;
  author: { name: string; url?: string };
  categories?: string[];
  permissions?: string[];
  icon_url?: string;
  download_url?: string;
  download_type?: "release" | "source";
  release_asset?: string;
  branch?: string;
  tags?: string[];
  dependencies?: string[];
  optional_dependencies?: string[];
  _path?: string;
}

export async function listPlugins(): Promise<PluginInfo[]> {
  return tauriInvoke("list_plugins");
}

export async function scanPlugins(): Promise<PluginInfo[]> {
  return tauriInvoke("scan_plugins");
}

export async function enablePlugin(pluginId: string): Promise<void> {
  return tauriInvoke("enable_plugin", { pluginId });
}

export async function disablePlugin(pluginId: string): Promise<string[]> {
  return tauriInvoke("disable_plugin", { pluginId });
}

export async function getPluginNavItems(): Promise<PluginNavItem[]> {
  return tauriInvoke("get_plugin_nav_items");
}

export async function installPlugin(path: string): Promise<PluginInstallResult> {
  return tauriInvoke("install_plugin", { path });
}

export async function installPluginsBatch(paths: string[]): Promise<BatchInstallResult> {
  return tauriInvoke("install_plugins_batch", { paths });
}

export async function getPluginIcon(pluginId: string): Promise<string> {
  return tauriInvoke("get_plugin_icon", { pluginId });
}

export async function getPluginSettings(pluginId: string): Promise<Record<string, unknown>> {
  return tauriInvoke("get_plugin_settings", { pluginId });
}

export async function setPluginSettings(
  pluginId: string,
  settings: Record<string, unknown>,
): Promise<void> {
  return tauriInvoke("set_plugin_settings", { pluginId, settings });
}

export async function getPluginCss(pluginId: string): Promise<string> {
  return tauriInvoke("get_plugin_css", { pluginId });
}

export async function getAllPluginCss(): Promise<[string, string][]> {
  return tauriInvoke("get_all_plugin_css");
}

export async function deletePlugin(pluginId: string, deleteData?: boolean): Promise<void> {
  return tauriInvoke("delete_plugin", { pluginId, deleteData });
}

export async function deletePlugins(pluginIds: string[], deleteData?: boolean): Promise<void> {
  return tauriInvoke("delete_plugins", { pluginIds, deleteData });
}

export async function checkPluginUpdate(pluginId: string): Promise<PluginUpdateInfo | null> {
  return tauriInvoke("check_plugin_update", { pluginId });
}

export async function checkAllPluginUpdates(): Promise<PluginUpdateInfo[]> {
  return tauriInvoke("check_all_plugin_updates");
}

export async function fetchMarketPlugins(marketUrl?: string): Promise<MarketPluginInfo[]> {
  return tauriInvoke("fetch_market_plugins", { marketUrl });
}

export async function fetchMarketPluginDetail(pluginPath: string): Promise<MarketPluginInfo> {
  return tauriInvoke("fetch_market_plugin_detail", { pluginPath });
}

export async function fetchMarketCategories(
  marketUrl?: string,
): Promise<Record<string, Record<string, string> | string>> {
  return tauriInvoke("fetch_market_categories", { marketUrl });
}

export interface InstallFromMarketOptions {
  pluginId: string;
  downloadUrl?: string;
  repo?: string;
  downloadType?: "release" | "source";
  releaseAsset?: string;
  branch?: string;
  version?: string;
}

export async function installFromMarket(
  options: InstallFromMarketOptions,
): Promise<PluginInstallResult> {
  return tauriInvoke("install_from_market", {
    pluginId: options.pluginId,
    downloadUrl: options.downloadUrl,
    repo: options.repo,
    downloadType: options.downloadType,
    releaseAsset: options.releaseAsset,
    branch: options.branch,
    version: options.version,
  });
}

export async function onLocaleChanged(locale: string): Promise<void> {
  return tauriInvoke("on_locale_changed", { locale });
}

export async function onPageChanged(path: string): Promise<void> {
  return tauriInvoke("on_page_changed", { path });
}

export async function componentMirrorClear(): Promise<void> {
  return tauriInvoke("component_mirror_clear");
}

export async function contextMenuCallback(
  pluginId: string,
  context: string,
  itemId: string,
  targetData: Record<string, unknown>,
): Promise<void> {
  return tauriInvoke("context_menu_callback", { pluginId, context, itemId, targetData });
}

export interface BufferedUiEvent {
  plugin_id: string;
  action: string;
  element_id: string;
  html: string;
}

export async function getPluginUiSnapshot(): Promise<BufferedUiEvent[]> {
  return tauriInvoke("get_plugin_ui_snapshot");
}

export interface BufferedSidebarEvent {
  plugin_id: string;
  action: string;
  label: string;
  icon: string;
}

export async function getPluginSidebarSnapshot(): Promise<BufferedSidebarEvent[]> {
  return tauriInvoke("get_plugin_sidebar_snapshot");
}

export interface BufferedContextMenuEvent {
  plugin_id: string;
  action: string;
  context: string;
  items: string;
}

export async function getPluginContextMenuSnapshot(): Promise<BufferedContextMenuEvent[]> {
  return tauriInvoke("get_plugin_context_menu_snapshot");
}

export interface BufferedComponentEvent {
  plugin_id: string;
  payload_json: string;
}

export async function getPluginComponentSnapshot(): Promise<BufferedComponentEvent[]> {
  return tauriInvoke("get_plugin_component_snapshot");
}
