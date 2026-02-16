import { defineStore } from "pinia";
import { i18n } from "../locales";

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
  },
  actions: {
    setLocale(locale: string) {
      this.locale = locale;
      i18n.setLocale(locale);
    },
    toggleLocale() {
      let newLocale = "zh-CN";
      if (this.locale === "zh-CN") {
        newLocale = "zh-TW";
      } else if (this.locale === "zh-TW") {
        newLocale = "en-US";
      }
      this.setLocale(newLocale);
    },
  },
});
