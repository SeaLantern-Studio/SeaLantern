import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import zhTW from "./zh-TW.json";
import { ref, type Ref } from "vue";

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

export const SUPPORTED_LOCALES = ["zh-CN", "en-US", "zh-TW"] as const;
export type LocaleCode = (typeof SUPPORTED_LOCALES)[number];

const translations: Record<LocaleCode, TranslationNode> = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "zh-TW": zhTW,
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
  private fallbackLocale: LocaleCode = "zh-CN";

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
