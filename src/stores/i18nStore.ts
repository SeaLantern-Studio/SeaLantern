import { defineStore } from 'pinia';
import { i18n } from '../locales';

export const useI18nStore = defineStore('i18n', {
  state: () => ({
    locale: 'zh-CN' as string
  }),
  getters: {
    currentLocale: (state) => state.locale,
    isChinese: (state) => state.locale === 'zh-CN' || state.locale === 'zh-TW',
    isSimplifiedChinese: (state) => state.locale === 'zh-CN',
    isTraditionalChinese: (state) => state.locale === 'zh-TW',
    isEnglish: (state) => state.locale === 'en-US',
    isDongbeiChinese: (state) => state.locale === 'zh-NE',
    isJapanese : (state) => state.locale === 'ja-JP',
    isUpsideDownEnglish : (state) => state.locale === 'en-UD',
    isChineseMeow : (state) => state.locale === 'zh-ME',
    isSpanish : (state) => state.locale === 'es-ES',
    isDeutsch : (state) => state.locale === 'de-DE',
    isPirate : (state) => state.locale === 'en-PT',
    isTianjin : (state) => state.locale === 'zh-TJ'
  },
  actions: {
    setLocale(locale: string) {
      this.locale = locale;
      i18n.setLocale(locale);
    },
    toggleLocale() {
      let newLocale = 'zh-CN';
      if (this.locale === 'zh-CN') {
        newLocale = 'zh-TW';
      } else if (this.locale === 'zh-TW') {
        newLocale = 'en-US';
      }
      this.setLocale(newLocale);
    }
  }
});