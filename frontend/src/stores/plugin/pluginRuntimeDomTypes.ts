import type { PluginUiAction } from "@type/plugin";

export interface PluginRuntimeUiEvent {
  plugin_id: string;
  action: PluginUiAction;
  element_id: string;
  html: string;
  target?: string;
}

export type PluginRuntimeEventListenerRegistry = Map<
  string,
  Array<{ element: Element; eventType: string; handler: EventListener }>
>;
