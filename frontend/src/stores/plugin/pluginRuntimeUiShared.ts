import type {
  PluginRuntimeEventListenerRegistry,
  PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDom";

export interface PluginRuntimeUiBridgeOptions {
  isBrowserEnv: () => boolean;
}

export type { PluginRuntimeEventListenerRegistry, PluginRuntimeUiEvent };
