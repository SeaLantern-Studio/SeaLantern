import { useToast } from "@composables/useToast";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginRuntimeUiEvent } from "@stores/plugin/pluginRuntimeDom";

export function handlePluginRuntimeFeedbackEvent(event: PluginRuntimeUiEvent): boolean {
  if (event.action !== "toast") {
    return false;
  }

  try {
    const { type, message, duration } = JSON.parse(event.html || "{}") as {
      type?: string;
      message?: string;
      duration?: number;
    };
    const validTypes = ["success", "error", "warning", "info"] as const;
    const toastType = validTypes.includes(type as (typeof validTypes)[number]) ? type : "info";
    const toast = useToast();
    (
      toast[toastType as "success" | "error" | "warning" | "info"] as (
        toastMessage: string,
        toastDuration?: number,
      ) => void
    )(message || "", duration);
  } catch (error) {
    pluginLogger.error("RuntimeUI", "插件提示内容无效", error);
  }

  return true;
}
