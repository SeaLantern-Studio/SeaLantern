import { handlePluginRuntimeFeedbackEvent } from "@stores/plugin/pluginRuntimeFeedback";
import {
  handlePluginRuntimeDomEvent,
  type PluginRuntimeEventListenerRegistry,
  type PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDom";

export function createPluginRuntimeUiHandler(
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
) {
  async function handlePluginUiEvent(event: PluginRuntimeUiEvent) {
    if (await handlePluginRuntimeDomEvent(event, eventListenerRegistry)) {
      return;
    }

    handlePluginRuntimeFeedbackEvent(event);
  }

  return {
    handlePluginUiEvent,
  };
}
