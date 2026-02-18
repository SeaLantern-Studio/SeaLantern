import { ref, type Ref } from "vue";
import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import zhTW from "./zh-TW.json";
import zhJB from "./zh-JB.json";
import zhNE from "./zh-NE.json";
import deDE from "./de-DE.json";
import enAU from "./en-AU.json";
import enGB from "./en-GB.json";
import enPT from "./en-PT.json";
import enUN from "./en-UD.json";
import esES from "./es-ES.json";
import jaJP from "./ja-JP.json";
import ruRU from "./ru-RU.json";
import viVN from "./vi-VN.json";
import zhCT from "./zh-CT.json";
import zhCY from "./zh-CY.json";
import zhHN from "./zh-HN.json";
import zhJL from "./zh-JL.json";
import zhME from "./zh-ME.json";
import zhMN from "./zh-MN.json";
import zhTJ from "./zh-TJ.json";
import zhWU from "./zh-WU.json";
import jaKS from "./ja-KS.json";
import jaHK from "./ja-HK.json";

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

export const SUPPORTED_LOCALES = [
  "zh-CN",
  "en-US",
  "zh-TW",
  "zh-JB",
  "zh-NE",
  "de-DE",
  "en-AU",
  "en-GB",
  "en-PT",
  "en-UN",
  "es-ES",
  "ja-JP",
  "ru-RU",
  "vi-VN",
  "zh-CT",
  "zh-CY",
  "zh-HN",
  "zh-JL",
  "zh-ME",
  "zh-MN",
  "zh-TJ",
  "zh-WU",
  "ja-KS",
  "ja-HK",
] as const;
export type LocaleCode = (typeof SUPPORTED_LOCALES)[number];

const translations: Record<LocaleCode, TranslationNode> = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "zh-TW": zhTW,
  "zh-JB": zhJB,
  "zh-NE": zhNE,
  "de-DE": deDE,
  "en-AU": enAU,
  "en-GB": enGB,
  "en-PT": enPT,
  "en-UN": enUN,
  "es-ES": esES,
  "ja-JP": jaJP,
  "ru-RU": ruRU,
  "vi-VN": viVN,
  "zh-CT": zhCT,
  "zh-CY": zhCY,
  "zh-HN": zhHN,
  "zh-JL": zhJL,
  "zh-ME": zhME,
  "zh-MN": zhMN,
  "zh-TJ": zhTJ,
  "zh-WU": zhWU,
  "ja-KS": jaKS,
  "ja-HK": jaHK,
};

function isSupportedLocale(locale: string): locale is LocaleCode {
  return (SUPPORTED_LOCALES as readonly string[]).includes(locale);
}

function resolveNestedValue(source: TranslationNode, keys: string[]): string | undefined {
  let current: string | TranslationNode | undefined = source;
  for (const key of keys) {
    if (!current || typeof current === "string") {
      return undefined;
    }
    current = current[key];
  }

  return typeof current === "string" ? current : undefined;
}

function interpolateVariables(template: string, options: Record<string, unknown>): string {
  // 同时支持 {{variable}} 和 {variable} 两种格式的占位符
  return template
    .replace(/\{\{([^}]+)\}\}/g, (match, varName) => {
      const value = options[varName.trim()];
      return value === undefined || value === null ? match : String(value);
    })
    .replace(/\{([^}]+)\}/g, (match, varName) => {
      const value = options[varName.trim()];
      return value === undefined || value === null ? match : String(value);
    });
}

class I18n {
  private currentLocale: Ref<LocaleCode> = ref("zh-CN");
  private fallbackLocale: LocaleCode = "en-US";

  setLocale(locale: string) {
    if (isSupportedLocale(locale)) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    const keys = key.split(".");
    const currentLocaleValue = this.currentLocale.value;
    const resolved =
      resolveNestedValue(translations[currentLocaleValue], keys) ??
      resolveNestedValue(translations[this.fallbackLocale], keys);

    if (resolved === undefined) {
      return key;
    }

    return interpolateVariables(resolved, options);
  }

  getTranslations() {
    return translations;
  }

  getLocaleRef() {
    return this.currentLocale;
  }

  getAvailableLocales(): readonly LocaleCode[] {
    return SUPPORTED_LOCALES;
  }

  isSupportedLocale(locale: string): boolean {
    return isSupportedLocale(locale);
  }
}

export const i18n = new I18n();
export default i18n;
