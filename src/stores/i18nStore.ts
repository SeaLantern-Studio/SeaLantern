import { defineStore } from "pinia";
import { i18n } from "../locales";

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

export const useI18nStore = defineStore("i18n", {
  state: () => ({
    locale: "zh-CN" as string,
  }),
  getters: {
    currentLocale: (state) => state.locale,
    isChinese: (state) => state.locale === "zh-CN" || state.locale === "zh-TW",
    isSimplifiedChinese: (state) => state.locale === "zh-CN",
    isTraditionalChinese: (state) => state.locale === "zh-TW",
    isEnglish: (state) => state.locale === "en-US",
    isJapanese: (state) => state.locale === "ja-JP",
    isFrench: (state) => state.locale === "fr-FR",
    isGerman: (state) => state.locale === "de-DE",
    isRussian: (state) => state.locale === "ru-RU",
    isArabic: (state) => state.locale === "ar-SA",
    isSpanish: (state) => state.locale === "es-ES",
    isItalian: (state) => state.locale === "it-IT",
    isPortuguese: (state) => state.locale === "pt-BR",
    isKorean: (state) => state.locale === "ko-KR",
    isDutch: (state) => state.locale === "nl-NL",
    isPolish: (state) => state.locale === "pl-PL",
    isTurkish: (state) => state.locale === "tr-TR",
    isVietnamese: (state) => state.locale === "vi-VN",
    availableLocales: () => AVAILABLE_LOCALES,
  },
  actions: {
    setLocale(locale: string) {
      this.locale = locale;
      i18n.setLocale(locale);
    },
    toggleLocale() {
      const currentIndex = AVAILABLE_LOCALES.indexOf(this.locale as any);
      const nextIndex = (currentIndex + 1) % AVAILABLE_LOCALES.length;
      this.setLocale(AVAILABLE_LOCALES[nextIndex]);
    },
  },
});
