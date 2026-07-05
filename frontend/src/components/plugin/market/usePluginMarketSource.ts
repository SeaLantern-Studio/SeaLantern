import { computed, ref } from "vue";
import { i18n } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { MarketFeedbackType } from "./pluginMarketShared";
import { MARKET_BASE_URL, MARKET_URL_KEY, validateMarketUrl } from "./pluginMarketShared";

interface UsePluginMarketSourceOptions {
  onSourceChanged: () => void;
  showFeedback: (type: MarketFeedbackType, message: string, duration?: number) => void;
}

export function usePluginMarketSource(options: UsePluginMarketSourceOptions) {
  const showUrlEditor = ref(false);
  const customMarketUrl = ref(localStorage.getItem(MARKET_URL_KEY) || "");
  const urlInput = ref(customMarketUrl.value);
  const pendingMarketSource = ref<string | null>(null);
  const showCustomSourceConfirm = ref(false);

  const validatedMarketSource = computed(() => {
    const validated = validateMarketUrl(customMarketUrl.value);
    if (validated) {
      return validated;
    }

    const fallback = new URL(MARKET_BASE_URL);
    return {
      url: MARKET_BASE_URL,
      custom: false,
      host: fallback.host,
      protocol: fallback.protocol,
    };
  });

  const activeMarketUrl = computed(() => validatedMarketSource.value.url);
  const activeMarketHost = computed(() => validatedMarketSource.value.host);
  const isUsingCustomMarket = computed(() => validatedMarketSource.value.custom);

  function toggleUrlEditor() {
    showUrlEditor.value = !showUrlEditor.value;
  }

  function applyMarketUrl(url: string) {
    const normalized = url.trim();
    customMarketUrl.value = normalized === MARKET_BASE_URL ? "" : normalized;
    urlInput.value = customMarketUrl.value;

    if (customMarketUrl.value) {
      localStorage.setItem(MARKET_URL_KEY, customMarketUrl.value);
    } else {
      localStorage.removeItem(MARKET_URL_KEY);
    }

    showUrlEditor.value = false;
    showCustomSourceConfirm.value = false;
    pendingMarketSource.value = null;

    pluginLogger.info("Market", "插件市场来源已更新", {
      source: customMarketUrl.value || MARKET_BASE_URL,
      custom: Boolean(customMarketUrl.value),
    });
    options.onSourceChanged();
  }

  function saveMarketUrl() {
    const url = urlInput.value.trim();
    if (!url) {
      resetMarketUrl();
      return;
    }

    const validated = validateMarketUrl(url);
    if (!validated) {
      options.showFeedback("error", i18n.t("market.source_invalid"));
      return;
    }

    if (validated.custom) {
      pendingMarketSource.value = validated.url;
      showCustomSourceConfirm.value = true;
      return;
    }

    applyMarketUrl(validated.url);
  }

  function confirmCustomMarketUrl() {
    if (!pendingMarketSource.value) {
      showCustomSourceConfirm.value = false;
      return;
    }

    applyMarketUrl(pendingMarketSource.value);
  }

  function cancelCustomMarketUrl() {
    pendingMarketSource.value = null;
    showCustomSourceConfirm.value = false;
  }

  function resetMarketUrl() {
    urlInput.value = "";
    customMarketUrl.value = "";
    pendingMarketSource.value = null;
    localStorage.removeItem(MARKET_URL_KEY);
    showUrlEditor.value = false;
    showCustomSourceConfirm.value = false;

    pluginLogger.info("Market", "插件市场来源已恢复默认", {
      source: MARKET_BASE_URL,
    });
    options.onSourceChanged();
  }

  return {
    showUrlEditor,
    customMarketUrl,
    urlInput,
    activeMarketUrl,
    activeMarketHost,
    isUsingCustomMarket,
    pendingMarketSource,
    showCustomSourceConfirm,
    saveMarketUrl,
    confirmCustomMarketUrl,
    cancelCustomMarketUrl,
    resetMarketUrl,
    toggleUrlEditor,
  };
}
