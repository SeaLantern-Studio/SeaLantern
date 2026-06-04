import {
  cleanupPluginEventListeners as cleanupRuntimeDomEventListeners,
  removePluginUiElements,
  sanitizeCss,
  type PluginRuntimeEventListenerRegistry,
} from "@stores/plugin/pluginRuntimeDom";
import { createPluginRuntimeUiHandler } from "./pluginRuntimeUiHandler";
import { createPluginRuntimeUiListeners } from "./pluginRuntimeUiListeners";
import type { PluginRuntimeUiBridgeOptions } from "./pluginRuntimeUiShared";

export function createPluginRuntimeUiBridge(options: PluginRuntimeUiBridgeOptions) {
  const eventListenerRegistry: PluginRuntimeEventListenerRegistry = new Map();
  const handler = createPluginRuntimeUiHandler(eventListenerRegistry);
  const listeners = createPluginRuntimeUiListeners({
    isBrowserEnv: options.isBrowserEnv,
    handlePluginUiEvent: handler.handlePluginUiEvent,
  });

  function cleanupPluginEventListeners(pluginId: string) {
    cleanupRuntimeDomEventListeners(pluginId, eventListenerRegistry);
  }

  return {
    sanitizeCss,
    handlePluginUiEvent: handler.handlePluginUiEvent,
    removePluginUiElements,
    cleanupPluginEventListeners,
    initUiEventListener: listeners.initUiEventListener,
    cleanupUiEventListener: listeners.cleanupUiEventListener,
    initSidebarEventListener: listeners.initSidebarEventListener,
    cleanupSidebarEventListener: listeners.cleanupSidebarEventListener,
  };
}
