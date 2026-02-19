import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import zhTW from "./zh-TW.json";
import jaJP from "./ja-JP.json";
import koKR from "./ko-KR.json";
import esES from "./es-ES.json";
import { ref, type Ref } from "vue";

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

export const SUPPORTED_LOCALES = ["zh-CN", "en-US", "zh-TW", "ja-JP", "ko-KR", "es-ES"] as const;
export type LocaleCode = (typeof SUPPORTED_LOCALES)[number];

const translations: Record<LocaleCode, TranslationNode> = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "zh-TW": zhTW,
  "ja-JP": jaJP,
  "ko-KR": koKR,
  "es-ES": esES,
};

function isSupportedLocale(locale: string): locale is LocaleCode {
  return (SUPPORTED_LOCALES as readonly string[]).includes(locale);
}

function normalizeLocale(input: string): string {
  return input.replace("_", "-").trim();
}

function resolveLocaleByPrefix(locale: string): LocaleCode | null {
  const normalized = normalizeLocale(locale);
  if (isSupportedLocale(normalized)) {
    return normalized;
  }

  const languagePart = normalized.split("-")[0]?.toLowerCase();
  if (!languagePart) return null;

  const matched = SUPPORTED_LOCALES.find((candidate) => candidate.toLowerCase().startsWith(`${languagePart}-`));
  return matched ?? null;
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
    const resolved = this.resolveLocale(locale);
    if (resolved) {
      this.currentLocale.value = resolved;
    }
  }

  resolveLocale(locale: string): LocaleCode | null {
    return resolveLocaleByPrefix(locale);
  }

  resolveBestLocale(candidates: readonly string[]): LocaleCode {
    for (const candidate of candidates) {
      const resolved = this.resolveLocale(candidate);
      if (resolved) {
        return resolved;
      }
    }

    return this.fallbackLocale;
  }

  detectSystemLocale(): LocaleCode {
    if (typeof navigator === "undefined") {
      return this.fallbackLocale;
    }

    return this.resolveBestLocale(navigator.languages ?? [navigator.language]);
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
    return this.resolveLocale(locale) !== null;
  }
}

export const i18n = new I18n();
export default i18n;
