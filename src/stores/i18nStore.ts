import { defineStore } from 'pinia';
import { i18n } from '../locales';

export const useI18nStore = defineStore('i18n', {
  state: () => ({
    locale: 'zh-CN' as string
  }),
  getters: {
    currentLocale: (state) => state.locale,
    isChinese: (state) => state.locale === 'zh-CN',
    isEnglish: (state) => state.locale === 'en-US'
  },
  actions: {
    setLocale(locale: string) {
      this.locale = locale;
      i18n.setLocale(locale);
    },
    toggleLocale() {
      const newLocale = this.locale === 'zh-CN' ? 'en-US' : 'zh-CN';
      this.setLocale(newLocale);
    }
  }
});