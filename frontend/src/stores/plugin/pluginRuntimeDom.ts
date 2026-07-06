import {
  cleanupPluginEventListeners,
  removePluginUiElements,
} from "@stores/plugin/pluginRuntimeDomCleanup";
import { handlePluginRuntimeElementAction } from "@stores/plugin/pluginRuntimeDomElementActions";
import { sanitizeCss, sanitizeHtml } from "@stores/plugin/pluginRuntimeDomSanitizer";
import { handlePluginRuntimeStructureAction } from "@stores/plugin/pluginRuntimeDomStructureActions";
import type {
  PluginRuntimeEventListenerRegistry,
  PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDomTypes";

export { cleanupPluginEventListeners, removePluginUiElements, sanitizeCss, sanitizeHtml };
export type { PluginRuntimeEventListenerRegistry, PluginRuntimeUiEvent };

export async function handlePluginRuntimeDomEvent(
  event: PluginRuntimeUiEvent,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
): Promise<boolean> {
  if (
    await handlePluginRuntimeStructureAction(event, eventListenerRegistry, {
      handlePluginRuntimeDomEvent,
    })
  ) {
    return true;
  }

  return handlePluginRuntimeElementAction(event, eventListenerRegistry);
}
