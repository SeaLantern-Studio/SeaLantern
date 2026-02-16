import { computed } from "vue";
import { defineStore } from "pinia";
import { i18n, type LocaleCode } from "../locales";

const LOCALE_LABEL_KEYS: Record<LocaleCode, string> = {
  "zh-CN": "header.chinese",
  "zh-TW": "header.chinese_tw",
  "en-US": "header.english",
};

export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const supportedLocales = i18n.getAvailableLocales();

  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(
    () => localeRef.value === "zh-CN" || localeRef.value === "zh-TW"
  );
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");
  const localeOptions = computed(() =>
    supportedLocales.map((code) => ({
      code,
      labelKey: LOCALE_LABEL_KEYS[code],
    }))
  );

  function setLocale(nextLocale: string) {
    i18n.setLocale(nextLocale);
  }

  function toggleLocale() {
    const currentIndex = supportedLocales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % supportedLocales.length;
    i18n.setLocale(supportedLocales[nextIndex]);
  }

  return {
    locale,
    currentLocale,
    isChinese,
    isSimplifiedChinese,
    isTraditionalChinese,
    isEnglish,
    localeOptions,
    setLocale,
    toggleLocale,
  };
});
