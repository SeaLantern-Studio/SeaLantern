import { ref, type Ref } from "vue";
import enUS from "./en-US.json";
import zhCN from "./zh-CN.json";

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

// 语言文件类型，包含语言名称字段
type LanguageFile = TranslationNode & {
  languageName?: string;
};

type LocaleModule = { default: LanguageFile };

const localeLoaders = import.meta.glob<LocaleModule>("./*.json");
const translations: Record<string, LanguageFile> = {
  "zh-CN": zhCN as LanguageFile,
  "en-US": enUS as LanguageFile,
};
const supportedLocales: string[] = Object.keys(localeLoaders)
  .map((path) => path.match(/\.\/(.*)\.json$/)?.[1])
  .filter((locale): locale is string => Boolean(locale));

const backendFlatTranslations: Record<string, Record<string, string>> = {};

const pluginFlatTranslations: Record<string, Record<string, Record<string, string>>> = {};
const pluginLocaleNames: Record<string, string> = {};

export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  if (isSupportedLocale(locale)) {
    translations[locale] = data;
  }
}

export async function ensureLocaleLoaded(locale: LocaleCode): Promise<boolean> {
  if (translations[locale]) {
    return true;
  }

  const loader = localeLoaders[`./${locale}.json`];
  if (!loader) {
    return false;
  }

  const module = await loader();
  translations[locale] = module.default;
  if (!supportedLocales.includes(locale)) {
    supportedLocales.push(locale);
  }
  return true;
}

export function setLocaleBundle(
  locale: LocaleCode,
  entries: Record<string, string>,
  locales?: readonly string[],
) {
  backendFlatTranslations[locale] = entries;
  if (!translations[locale]) {
    translations[locale] = {};
  }
  if (locales) {
    for (const localeCode of locales) {
      if (!supportedLocales.includes(localeCode)) {
        supportedLocales.push(localeCode);
      }
    }
  }
}

export function registerPluginLocale(locale: string, displayName: string) {
  pluginLocaleNames[locale] = displayName;
  if (!supportedLocales.includes(locale)) {
    supportedLocales.push(locale);
  }
}

export function addPluginTranslations(
  pluginId: string,
  locale: string,
  entries: Record<string, string>,
) {
  if (!pluginFlatTranslations[pluginId]) {
    pluginFlatTranslations[pluginId] = {};
  }
  if (!pluginFlatTranslations[pluginId][locale]) {
    pluginFlatTranslations[pluginId][locale] = {};
  }
  Object.assign(pluginFlatTranslations[pluginId][locale], entries);
}

export function removePluginTranslations(pluginId: string) {
  delete pluginFlatTranslations[pluginId];
}

export function getPluginLocaleDisplayName(locale: string): string | undefined {
  return pluginLocaleNames[locale];
}

function isSupportedLocale(locale: string): locale is LocaleCode {
  return supportedLocales.includes(locale);
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

function resolveBackendFlatValue(locale: string, key: string): string | undefined {
  return backendFlatTranslations[locale]?.[key];
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
    if (isSupportedLocale(locale) || pluginLocaleNames[locale] !== undefined) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    const keys = key.split(".");
    const currentLocaleValue = this.currentLocale.value;

    let resolved: string | undefined =
      resolveBackendFlatValue(currentLocaleValue, key) ??
      resolveBackendFlatValue(currentLocaleValue, `sealantern.${key}`) ??
      resolveNestedValue(translations[currentLocaleValue], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[currentLocaleValue], keys) ??
      resolveBackendFlatValue(this.fallbackLocale, key) ??
      resolveBackendFlatValue(this.fallbackLocale, `sealantern.${key}`) ??
      resolveNestedValue(translations[this.fallbackLocale], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[this.fallbackLocale], keys);

    if (resolved === undefined) {
      for (const pluginMap of Object.values(pluginFlatTranslations)) {
        const val = pluginMap[currentLocaleValue]?.[key] ?? pluginMap[this.fallbackLocale]?.[key];
        if (val !== undefined) {
          resolved = val;
          break;
        }
      }
    }

    if (resolved === undefined) {
      return key;
    }

    return interpolateVariables(resolved, options);
  }

  te(key: string): boolean {
    const keys = key.split(".");
    const currentLocaleValue = this.currentLocale.value;
    const resolved =
      resolveBackendFlatValue(currentLocaleValue, key) ??
      resolveBackendFlatValue(currentLocaleValue, `sealantern.${key}`) ??
      resolveNestedValue(translations[currentLocaleValue], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[currentLocaleValue], keys) ??
      resolveBackendFlatValue(this.fallbackLocale, key) ??
      resolveBackendFlatValue(this.fallbackLocale, `sealantern.${key}`) ??
      resolveNestedValue(translations[this.fallbackLocale], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[this.fallbackLocale], keys);
    return resolved !== undefined;
  }

  getTranslations() {
    return translations as Record<string, LanguageFile>;
  }

  getLocaleRef() {
    return this.currentLocale;
  }

  getAvailableLocales(): readonly LocaleCode[] {
    return supportedLocales;
  }

  isSupportedLocale(locale: string): boolean {
    return (SUPPORTED_LOCALES as readonly string[]).includes(locale);
  }
}

export const i18n = new I18n();

const languageAPI = {
  i18n,
  SUPPORTED_LOCALES,
  setTranslations,
  ensureLocaleLoaded,
  setLocaleBundle,
  registerPluginLocale,
  addPluginTranslations,
  removePluginTranslations,
  getPluginLocaleDisplayName,
};

export default languageAPI;
