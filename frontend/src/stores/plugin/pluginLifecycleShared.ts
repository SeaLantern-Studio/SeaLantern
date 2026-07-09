import type {
  MissingDependency,
  PluginEnableBlockReason,
  PluginEnableGrantScope,
  PluginInfo,
  PluginNavItem,
  PluginUpdateInfo,
} from "@type/plugin";
import type { Ref } from "vue";

export interface CreatePluginLifecycleActionsOptions {
  plugins: Ref<PluginInfo[]>;
  navItems: Ref<PluginNavItem[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  icons: Ref<Record<string, string>>;
  updates: Ref<Record<string, PluginUpdateInfo>>;
  pendingDependencies: Ref<MissingDependency[]>;
  syncThemeProviderOverrides: (pluginIds?: string[]) => void;
  loadPluginIcons: () => Promise<void>;
  injectAllPluginCss: () => Promise<void>;
  injectPluginCss: (pluginId: string) => Promise<void>;
  removePluginCss: (pluginId: string) => void;
  removePluginUiElements: (pluginId: string) => void;
  cleanupPluginEventListeners: (pluginId: string) => void;
  removePluginProxies: (pluginId: string) => void;
  removePluginComponents: (pluginId: string) => void;
  replayUiSnapshot: () => Promise<void>;
  collectSidebarItems: () => void;
}

export interface TogglePluginResult {
  success: boolean;
  error?: string;
  disabledPlugins?: string[];
  confirmationRequired?: boolean;
  blockReason?: PluginEnableBlockReason;
  grantScope?: PluginEnableGrantScope;
  plugin?: PluginInfo | null;
  message?: string | null;
}

export type SetStoreError = (message: string, errorCause: unknown) => string;

export function currentPluginPath() {
  return window.location.hash.replace(/^#/, "") || "/";
}

export function setPluginState(
  options: Pick<CreatePluginLifecycleActionsOptions, "plugins">,
  pluginId: string,
  state: PluginInfo["state"],
) {
  const pluginIndex = options.plugins.value.findIndex((plugin) => plugin.manifest.id === pluginId);
  if (pluginIndex !== -1) {
    options.plugins.value[pluginIndex].state = state;
  }
}

export function cleanupPluginRuntime(
  options: Pick<
    CreatePluginLifecycleActionsOptions,
    | "removePluginCss"
    | "removePluginUiElements"
    | "cleanupPluginEventListeners"
    | "removePluginProxies"
    | "removePluginComponents"
  >,
  pluginId: string,
) {
  options.removePluginCss(pluginId);
  options.removePluginUiElements(pluginId);
  options.cleanupPluginEventListeners(pluginId);
  options.removePluginProxies(pluginId);
  options.removePluginComponents(pluginId);
}

export function clearPluginArtifacts(
  options: Pick<CreatePluginLifecycleActionsOptions, "icons" | "updates" | "removePluginCss">,
  pluginId: string,
) {
  delete options.icons.value[pluginId];
  delete options.updates.value[pluginId];
  options.removePluginCss(pluginId);
}
