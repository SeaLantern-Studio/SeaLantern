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

const LOCALE_LABEL_KEYS: Record<string, string> = {
  "zh-CN": "header.chinese",
  "en-US": "header.english",
  "zh-TW": "header.chinese_tw",
  "de-DE": "header.deutsch",
  "es-ES": "header.spanish",
  "ja-JP": "header.japanese",
  "ru-RU": "header.russian",
  "vi-VN": "header.vietnamese",
  "ko-KR": "header.korean",
  "fr-FA": "header.french",
};

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
  } catch (e) {
    console.error("Failed to load locale:", localeCode, e);
  }
}

export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const supportedLocales = i18n.getAvailableLocales();
  const settingsStore = useSettingsStore();

  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(() => localeRef.value === "zh-CN" || localeRef.value === "zh-TW");
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");
  const localeOptions = computed(() =>
    supportedLocales.map((code) => ({
      code,
      labelKey: LOCALE_LABEL_KEYS[code],
    })),
  );
  async function setLocale(nextLocale: string) {
    const localeReady = await ensureLocaleReady(nextLocale);
    if (!localeReady) {
      return false;
    }

    i18n.setLocale(nextLocale);

    try {
      await settingsStore.setLanguage(nextLocale);
    } catch (error) {
      console.error("Failed to save language setting:", normalizeAppError(error));
    }

    try {
      await onLocaleChanged(nextLocale);
    } catch (error) {
      console.error("Failed to notify backend about locale change:", normalizeAppError(error));
    }

    return true;
  }

  function toggleLocale() {
    const currentIndex = supportedLocales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % supportedLocales.length;
    void setLocale(supportedLocales[nextIndex]);
  }

  async function loadLanguageSetting() {
    try {
      await settingsStore.ensureLoaded();
      const language = settingsStore.settings.language;
      if (language && (await ensureLocaleReady(language))) {
        i18n.setLocale(language);
      }
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
    localeOptions,
    ensureLocaleReady,
    setLocale,
    toggleLocale,
    loadLanguageSetting,
    downloadLocale,
  };
});
