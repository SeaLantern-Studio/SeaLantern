import { computed } from "vue";
import { defineStore } from "pinia";
import {
  ensureLocaleLoaded,
  i18n,
  type LocaleCode,
  setLocaleBundle,
  setTranslations,
} from "@language";
import { fetchLocale } from "@api/remoteLocales";
import { onLocaleChanged } from "@api/plugin";
import { tryLoadLocaleBundle } from "@composables/useI18nBundles";
import { useSettingsStore } from "@stores/settingsStore";
import { normalizeAppError, resolveErrorMessage } from "@utils/appError";

const DEFAULT_SELECTED_LOCALE = "zh-CN";

async function ensureLocaleReady(nextLocale: string): Promise<boolean> {
  if (!i18n.isSupportedLocale(nextLocale)) {
    return false;
  }

  const bundleLoaded = await tryLoadLocaleBundle(nextLocale);
  if (bundleLoaded) {
    return true;
  }

  return ensureLocaleLoaded(nextLocale as LocaleCode);
}

async function downloadLocale(localeCode: string) {
  if (!i18n.isSupportedLocale(localeCode)) return;
  try {
    const data = await fetchLocale(localeCode as LocaleCode);
    if (data && typeof data === "object" && !("sealantern" in data)) {
      setLocaleBundle(localeCode, data as Record<string, string>);
    } else {
      setTranslations(localeCode as LocaleCode, data as any);
    }
  } catch (error) {
    console.error("Failed to load locale:", localeCode, error);
  }
}

export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const settingsStore = useSettingsStore();

  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(() => localeRef.value === "zh-CN" || localeRef.value === "zh-TW");
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");

  const registeredLocaleOptions = computed(() => {
    return i18n.getAvailableLocales().map((code) => ({
      code,
      label: i18n.getLocaleDisplayName(code) ?? code,
    }));
  });

  const localeOptions = computed(() => registeredLocaleOptions.value);
  async function persistLanguageState(current: string): Promise<void> {
    await settingsStore.updatePartial({
      language: current,
      locale_layer_order: [],
    });
  }

  async function setLocale(nextLocale: string) {
    const localeReady = await ensureLocaleReady(nextLocale);
    if (!localeReady) {
      return false;
    }

    i18n.setLocale(nextLocale);

    try {
      await persistLanguageState(nextLocale);
    } catch (error) {
      console.error("Failed to save locale:", normalizeAppError(error));
      return false;
    }

    try {
      await onLocaleChanged(nextLocale);
    } catch (error) {
      console.error("Failed to notify backend about locale change:", normalizeAppError(error));
    }

    return true;
  }

  function toggleLocale() {
    const locales = registeredLocaleOptions.value.map((option) => option.code);
    const currentIndex = locales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % locales.length;
    const nextLocale = locales[nextIndex];
    if (nextLocale) {
      void setLocale(nextLocale);
    }
  }

  async function loadLanguageSetting() {
    try {
      await settingsStore.ensureLoaded();

      const savedLanguage = settingsStore.settings.language || DEFAULT_SELECTED_LOCALE;
      const selectedLocale = (await ensureLocaleReady(savedLanguage))
        ? savedLanguage
        : DEFAULT_SELECTED_LOCALE;
      await ensureLocaleReady(selectedLocale);

      i18n.setLocale(selectedLocale);
    } catch (error) {
      console.error("Failed to load language setting:", {
        error: normalizeAppError(error),
        fallback: resolveErrorMessage("common.message_unknown_error"),
      });
    }
  }

  return {
    locale,
    currentLocale,
    isChinese,
    isSimplifiedChinese,
    isTraditionalChinese,
    isEnglish,
    registeredLocaleOptions,
    localeOptions,
    ensureLocaleReady,
    setLocale,
    toggleLocale,
    loadLanguageSetting,
    downloadLocale,
  };
});
