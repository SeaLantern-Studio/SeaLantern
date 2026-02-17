import { computed, onMounted } from "vue";
import { defineStore } from "pinia";
import { i18n, type LocaleCode } from "../locales";
import { settingsApi } from "../api/settings";

import { computed } from "vue";
import { defineStore } from "pinia";
import { i18n, type LocaleCode } from "../locales";


const AVAILABLE_LOCALES = [
  "zh-CN",
  "zh-TW",
  "en-US",
  "ja-JP",
  "fr-FR",
  "de-DE",
  "ru-RU",
  "ar-SA",
  "es-ES",
  "it-IT",
  "pt-BR",
  "ko-KR",
  "nl-NL",
  "pl-PL",
  "tr-TR",
  "vi-VN",
] as const;

const LOCALE_LABEL_KEYS: Record<LocaleCode, string> = {
  "zh-CN": "header.chinese",
  "en-US": "header.english",
  "ja-JP": "header.japanese",
  "fr-FR": "header.french",
  "de-DE": "header.german",
  "ru-RU": "header.russian",
  "ar-SA": "header.arabic",
  "es-ES": "header.spanish",
  "it-IT": "header.italian",
  "pt-BR": "header.portuguese_br",
  "ko-KR": "header.korean",
  "nl-NL": "header.dutch",
  "pl-PL": "header.polish",
  "tr-TR": "header.turkish",
  "vi-VN": "header.vietnamese",
  "zh-TW": "header.chinese_tw",
  "zh-JB": "header.chinese_jb",
  "zh-NE": "header.chinese_dongbei",
  "de-DE": "header.deutsch",
  "en-AU": "header.aussie",
  "en-GB": "header.british",
  "en-PT": "header.pirate",
  "en-UN": "header.upsidedown",
  "es-ES": "header.spanish",
  "ja-JP": "header.japanese",
  "ru-RU": "header.russian",
  "vi-VN": "header.vietnamese",
  "zh-CT": "header.cantonese",
  "zh-CY": "header.chinese_cy",
  "zh-HN": "header.chinese_hn",
  "zh-JL": "header.chinese_jl",
  "zh-ME": "header.chinese_meow",
  "zh-MN": "header.chinese_hokkien",
  "zh-TJ": "header.chinese_tj",
  "zh-WU": "header.chinese_wu",
  "ja-KS": "header.kansaiben",
  "ja-HK": "header.hokkaidou"
};
export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const supportedLocales = i18n.getAvailableLocales();
  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(() => localeRef.value === "zh-CN" || localeRef.value === "zh-TW");
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");
  const isJapanese = computed(() => localeRef.value === "ja-JP");
  const isFrench = computed(() => localeRef.value === "fr-FR");
  const isGerman = computed(() => localeRef.value === "de-DE");
  const isRussian = computed(() => localeRef.value === "ru-RU");
  const isArabic = computed(() => localeRef.value === "ar-SA");
  const isSpanish = computed(() => localeRef.value === "es-ES");
  const isItalian = computed(() => localeRef.value === "it-IT");
  const isPortuguese = computed(() => localeRef.value === "pt-BR");
  const isKorean = computed(() => localeRef.value === "ko-KR");
  const isDutch = computed(() => localeRef.value === "nl-NL");
  const isPolish = computed(() => localeRef.value === "pl-PL");
  const isTurkish = computed(() => localeRef.value === "tr-TR");
  const isVietnamese = computed(() => localeRef.value === "vi-VN");
  const availableLocales = computed(() => AVAILABLE_LOCALES);
  const localeOptions = computed(() =>
    supportedLocales.map((code) => ({
      code,
      labelKey: LOCALE_LABEL_KEYS[code],
    })),
  );

  async function setLocale(nextLocale: string) {
    if (i18n.isSupportedLocale(nextLocale)) {
      i18n.setLocale(nextLocale);
      // 保存语言设置到持久化存储
      try {
        const settings = await settingsApi.get();
        settings.language = nextLocale;
        await settingsApi.save(settings);
      } catch (error) {
        console.error("Failed to save language setting:", error);
      }
    }
  }

  function toggleLocale() {
    const currentIndex = supportedLocales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % supportedLocales.length;
    setLocale(supportedLocales[nextIndex]);
  }

  // 从持久化存储加载语言设置
  async function loadLanguageSetting() {
    try {
      const settings = await settingsApi.get();
      if (settings.language && i18n.isSupportedLocale(settings.language)) {
        i18n.setLocale(settings.language);
      }
    } catch (error) {
      console.error("Failed to load language setting:", error);
    }
  }

  // 组件挂载时加载语言设置
  onMounted(() => {
    loadLanguageSetting();
  });

  return {
    locale,
    currentLocale,
    isChinese,
    isSimplifiedChinese,
    isTraditionalChinese,
    isEnglish,
    isJapanese,
    isFrench,
    isGerman,
    isRussian,
    isArabic,
    isSpanish,
    isItalian,
    isPortuguese,
    isKorean,
    isDutch,
    isPolish,
    isTurkish,
    isVietnamese,
    availableLocales,
    localeOptions,
    setLocale,
    toggleLocale,
    loadLanguageSetting,
  };
});
});
