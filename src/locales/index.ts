import zhCN from './zh-CN.json';
import enUS from './en-US.json';

interface Translation {
  [key: string]: any;
}

interface Translations {
  [locale: string]: Translation;
}

const translations: Translations = {
  'zh-CN': zhCN,
  'en-US': enUS
};

import { ref } from 'vue';

class I18n {
  private currentLocale = ref('zh-CN');
  private fallbackLocale = 'zh-CN';

  constructor() {
  }

  setLocale(locale: string) {
    if (translations[locale]) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): string {
    return this.currentLocale.value;
  }

  t(key: string, defaultValue: string = key): string {
    // Access currentLocale.value to establish reactive dependency
    const currentLocaleValue = this.currentLocale.value;
    const keys = key.split('.');
    let value: any = translations[currentLocaleValue];

    for (const k of keys) {
      if (value && typeof value === 'object' && k in value) {
        value = value[k];
      } else {
        // Try fallback locale
        value = translations[this.fallbackLocale];
        for (const k of keys) {
          if (value && typeof value === 'object' && k in value) {
            value = value[k];
          } else {
            return defaultValue;
          }
        }
      }
    }

    return typeof value === 'string' ? value : defaultValue;
  }

  getTranslations(): Translations {
    return translations;
  }

  getLocaleRef() {
    return this.currentLocale;
  }
}

export const i18n = new I18n();
export default i18n;