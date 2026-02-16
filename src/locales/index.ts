import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import zhTW from "./zh-TW.json";
import jaJP from "./ja-JP.json";
import frFR from "./fr-FR.json";
import deDE from "./de-DE.json";
import ruRU from "./ru-RU.json";
import arSA from "./ar-SA.json";
import esES from "./es-ES.json";
import itIT from "./it-IT.json";
import ptBR from "./pt-BR.json";
import koKR from "./ko-KR.json";
import nlNL from "./nl-NL.json";
import plPL from "./pl-PL.json";
import trTR from "./tr-TR.json";
import viVN from "./vi-VN.json";

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
  "ja-JP": jaJP,
  "fr-FR": frFR,
  "de-DE": deDE,
  "ru-RU": ruRU,
  "ar-SA": arSA,
  "es-ES": esES,
  "it-IT": itIT,
  "pt-BR": ptBR,
  "ko-KR": koKR,
  "nl-NL": nlNL,
  "pl-PL": plPL,
  "tr-TR": trTR,
  "vi-VN": viVN,
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
