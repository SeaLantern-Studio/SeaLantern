import { pluginLogger } from "@stores/plugin/pluginLogger";
import { type PluginRuntimeEventListenerRegistry } from "@stores/plugin/pluginRuntimeDom";

export function removePluginUiElements(pluginId: string) {
  const elements = document.querySelectorAll(`[data-plugin-id="${pluginId}"]`);
  elements.forEach((element) => element.remove());

  const insertedElements = document.querySelectorAll(`[data-plugin-inserted="${pluginId}"]`);
  insertedElements.forEach((element) => element.remove());

  const hiddenElements = document.querySelectorAll(`[data-plugin-hidden="${pluginId}"]`);
  hiddenElements.forEach((element) => {
    (element as HTMLElement).style.display = "";
    delete (element as HTMLElement).dataset.pluginHidden;
  });

  const disabledElements = document.querySelectorAll(`[data-plugin-disabled="${pluginId}"]`);
  disabledElements.forEach((element) => {
    (element as HTMLElement).removeAttribute("disabled");
    (element as HTMLElement).style.pointerEvents = "";
    (element as HTMLElement).style.opacity = "";
    delete (element as HTMLElement).dataset.pluginDisabled;
  });

  pluginLogger.info("RuntimeUI", `已清理插件界面元素: ${pluginId}`, {
    removed: elements.length + insertedElements.length,
  });
}

export function cleanupPluginEventListeners(
  pluginId: string,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
) {
  const listeners = eventListenerRegistry.get(pluginId);
  if (!listeners) {
    return;
  }

  for (const { element, eventType, handler } of listeners) {
    element.removeEventListener(eventType, handler);
  }
  eventListenerRegistry.delete(pluginId);
  pluginLogger.info("RuntimeUI", `已清理插件事件监听: ${pluginId}`, {
    count: listeners.length,
  });
}
