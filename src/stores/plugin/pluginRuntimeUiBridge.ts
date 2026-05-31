import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  cleanupPluginEventListeners as cleanupRuntimeDomEventListeners,
  handlePluginRuntimeDomEvent,
  removePluginUiElements,
  sanitizeCss,
  type PluginRuntimeEventListenerRegistry,
  type PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDom";
import { handlePluginRuntimeFeedbackEvent } from "@stores/plugin/pluginRuntimeFeedback";

interface PluginRuntimeUiBridgeOptions {
  isBrowserEnv: () => boolean;
}

export function createPluginRuntimeUiBridge(options: PluginRuntimeUiBridgeOptions) {
  const eventListenerRegistry: PluginRuntimeEventListenerRegistry = new Map();
  let uiEventUnlisten: UnlistenFn | null = null;
  let sidebarEventUnlisten: UnlistenFn | null = null;

  async function handlePluginUiEvent(event: PluginRuntimeUiEvent) {
    if (await handlePluginRuntimeDomEvent(event, eventListenerRegistry)) {
      return;
    }

    handlePluginRuntimeFeedbackEvent(event);
  }

  function cleanupPluginEventListeners(pluginId: string) {
    cleanupRuntimeDomEventListeners(pluginId, eventListenerRegistry);
  }

  async function initUiEventListener() {
    if (options.isBrowserEnv() || uiEventUnlisten) {
      return;
    }

    try {
      uiEventUnlisten = await listen<PluginRuntimeUiEvent>("plugin-ui-event", (event) => {
        void handlePluginUiEvent(event.payload);
      });
      pluginLogger.info("RuntimeUI", "界面事件监听已就绪");
    } catch (error) {
      pluginLogger.error("RuntimeUI", "界面事件监听初始化失败", error);
    }
  }

  function cleanupUiEventListener() {
    uiEventUnlisten?.();
    uiEventUnlisten = null;
  }

  async function initSidebarEventListener() {
    if (options.isBrowserEnv() || sidebarEventUnlisten) {
      return;
    }

    pluginLogger.debug("Sidebar", "侧边栏事件监听当前未启用");
  }

  function cleanupSidebarEventListener() {
    sidebarEventUnlisten?.();
    sidebarEventUnlisten = null;
  }

  return {
    sanitizeCss,
    handlePluginUiEvent,
    removePluginUiElements,
    cleanupPluginEventListeners,
    initUiEventListener,
    cleanupUiEventListener,
    initSidebarEventListener,
    cleanupSidebarEventListener,
  };
}
