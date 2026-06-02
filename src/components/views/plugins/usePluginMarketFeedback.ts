import type { Ref } from "vue";
import type { MarketFeedback, MarketFeedbackType } from "./pluginMarketShared";

interface UsePluginMarketFeedbackOptions {
  installFeedback: Ref<MarketFeedback | null>;
}

export function usePluginMarketFeedback(options: UsePluginMarketFeedbackOptions) {
  function showFeedback(type: MarketFeedbackType, message: string, duration = 6000) {
    options.installFeedback.value = { type, message };
    if (duration > 0) {
      setTimeout(() => {
        if (options.installFeedback.value?.message === message) {
          options.installFeedback.value = null;
        }
      }, duration);
    }
  }

  function clearFeedback() {
    options.installFeedback.value = null;
  }

  return {
    showFeedback,
    clearFeedback,
  };
}
