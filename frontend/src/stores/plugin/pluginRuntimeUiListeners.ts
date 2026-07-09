import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginRuntimeUiBridgeOptions, PluginRuntimeUiEvent } from "./pluginRuntimeUiShared";

interface PluginRuntimeUiListenersOptions extends PluginRuntimeUiBridgeOptions {
  handlePluginUiEvent: (event: PluginRuntimeUiEvent) => Promise<void>;
}

export function createPluginRuntimeUiListeners(options: PluginRuntimeUiListenersOptions) {
  let uiEventUnlisten: UnlistenFn | null = null;
  let sidebarEventUnlisten: UnlistenFn | null = null;

  async function initUiEventListener() {
    if (options.isBrowserEnv() || uiEventUnlisten) {
      return;
    }

    try {
      uiEventUnlisten = await listen<PluginRuntimeUiEvent>("plugin-ui-event", (event) => {
        void options.handlePluginUiEvent(event.payload);
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
    initUiEventListener,
    cleanupUiEventListener,
    initSidebarEventListener,
    cleanupSidebarEventListener,
  };
}
