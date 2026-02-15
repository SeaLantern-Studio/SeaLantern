import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import zhTW from "./zh-TW.json";

interface Translation {
  [key: string]: any;
}

interface Translations {
  [locale: string]: Translation;
}

const translations: Translations = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "zh-TW": zhTW,
};

import { ref } from "vue";

class I18n {
  private currentLocale = ref("zh-CN");
  private fallbackLocale = "zh-CN";

  

  setLocale(locale: string) {
    if (translations[locale]) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): string {
    return this.currentLocale.value;
  }

  t(key: string, options: any = {}): string {
    // Access currentLocale.value to establish reactive dependency
    const currentLocaleValue = this.currentLocale.value;
    const keys = key.split(".");
    let value: any = translations[currentLocaleValue];

    for (const k of keys) {
      if (value && typeof value === "object" && k in value) {
        value = value[k];
      } else {
        // Try fallback locale
        value = translations[this.fallbackLocale];
        for (const k of keys) {
          if (value && typeof value === "object" && k in value) {
            value = value[k];
          } else {
            return key;
          }
        }
      }
    }

    let result = typeof value === "string" ? value : key;

    // Replace variables like {{count}} with values from options
    if (typeof result === "string" && typeof options === "object" && options !== null) {
      result = result.replace(/\{\{([^}]+)\}\}/g, (match, varName) => {
        const trimmedVarName = varName.trim();
        return options[trimmedVarName] !== undefined ? options[trimmedVarName] : match;
      });
    }

    return result;
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
