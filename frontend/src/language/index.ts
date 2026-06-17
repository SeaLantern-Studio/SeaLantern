import { ref, type Ref } from "vue";

type TranslationValue = string | TranslationNode;

type TranslationNode = {
  [key: string]: TranslationValue;
};

export type LanguageFile = TranslationNode;

type LocaleAuthor = {
  name: string;
  email?: string;
};

export type LocaleMetadata = {
  language: string;
  isBuiltin?: boolean;
  authors?: LocaleAuthor[];
};

type LocaleModule = { default: unknown };
type LocaleMetadataModule = { default: LocaleMetadata };

const localeModules = import.meta.glob<LocaleModule>("./*/*.json", { eager: true });
const builtInLocaleMetadata: Record<string, LocaleMetadata> = {};
const builtInLocaleGroups: Record<string, Record<string, TranslationValue>> = {};

for (const [path, module] of Object.entries(localeModules)) {
  const match = path.match(/^\.\/([^/]+)\/([^/]+)\.json$/);
  if (!match) {
    continue;
  }

  const [, locale, group] = match;
  if (group === "language") {
    builtInLocaleMetadata[locale] = (module as LocaleMetadataModule).default;
    continue;
  }

  if (!builtInLocaleGroups[locale]) {
    builtInLocaleGroups[locale] = {};
  }
  builtInLocaleGroups[locale][group] = normalizeGroupValue(module.default);
}

const translations: Record<string, LanguageFile> = {};
const supportedLocales: string[] = Array.from(
  new Set([...Object.keys(builtInLocaleGroups), ...Object.keys(builtInLocaleMetadata)]),
).sort((a, b) => a.localeCompare(b));

for (const locale of Object.keys(builtInLocaleGroups)) {
  const localeTranslations = buildLocaleTranslations(locale);
  if (localeTranslations) {
    translations[locale] = localeTranslations;
  }
}

const backendFlatTranslations: Record<string, Record<string, string>> = {};

const pluginFlatTranslations: Record<string, Record<string, Record<string, string>>> = {};
const pluginLocaleNames: Record<string, string> = {};

export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  translations[locale] = normalizeLanguageFile(data);
  if (!supportedLocales.includes(locale)) {
    supportedLocales.push(locale);
  }
}

export async function ensureLocaleLoaded(locale: LocaleCode): Promise<boolean> {
  if (translations[locale]) {
    return true;
  }

  const localeTranslations = buildLocaleTranslations(locale);
  if (!localeTranslations) {
    return false;
  }

  translations[locale] = localeTranslations;
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

function isTranslationNode(value: unknown): value is TranslationNode {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function cloneTranslationValue(value: TranslationValue): TranslationValue {
  if (typeof value === "string") {
    return value;
  }

  const cloned: TranslationNode = {};
  for (const [key, child] of Object.entries(value)) {
    cloned[key] = cloneTranslationValue(child);
  }
  return cloned;
}

function normalizeGroupValue(value: unknown): TranslationValue {
  if (typeof value === "string") {
    return value;
  }

  if (!isTranslationNode(value)) {
    return {};
  }

  const normalized: TranslationNode = {};
  for (const [key, child] of Object.entries(value)) {
    normalized[key] = normalizeGroupValue(child);
  }
  return normalized;
}

function mergeTranslationValues(
  base: TranslationValue | undefined,
  override: TranslationValue | undefined,
): TranslationValue | undefined {
  if (override === undefined) {
    return base === undefined ? undefined : cloneTranslationValue(base);
  }

  if (base === undefined) {
    return cloneTranslationValue(override);
  }

  if (typeof base === "string" || typeof override === "string") {
    return cloneTranslationValue(override);
  }

  const merged: TranslationNode = {};
  const keys = new Set([...Object.keys(base), ...Object.keys(override)]);
  for (const key of keys) {
    const value = mergeTranslationValues(base[key], override[key]);
    if (value !== undefined) {
      merged[key] = value;
    }
  }
  return merged;
}

function normalizeLanguageFile(data: LanguageFile): LanguageFile {
  const source = isTranslationNode(data.sealantern) ? data.sealantern : data;
  const normalized: TranslationNode = {};

  for (const [key, value] of Object.entries(source)) {
    normalized[key] = normalizeGroupValue(value);
  }

  if (isTranslationNode(data.sealantern)) {
    for (const [key, value] of Object.entries(data)) {
      if (key === "sealantern") {
        continue;
      }
      const merged = mergeTranslationValues(normalized[key], normalizeGroupValue(value));
      if (merged !== undefined) {
        normalized[key] = merged;
      }
    }
  }

  return normalized;
}

function buildLocaleTranslations(locale: string): LanguageFile | undefined {
  const groups = builtInLocaleGroups[locale];
  if (!groups) {
    return undefined;
  }

  const localeTranslations: LanguageFile = {};
  for (const [group, value] of Object.entries(groups)) {
    localeTranslations[group] = cloneTranslationValue(value);
  }
  return localeTranslations;
}

function resolveNestedValue(source: TranslationNode | undefined, keys: string[]): string | undefined {
  let current: TranslationValue | undefined = source;
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

function getLocaleFallbackChain(locale: string): LocaleCode[] {
  return Array.from(new Set([locale, "en-US", "zh-CN"]));
}

function resolveLocaleValue(locale: string, key: string): string | undefined {
  const keys = key.split(".");
  return (
    resolveBackendFlatValue(locale, key) ??
    resolveBackendFlatValue(locale, `sealantern.${key}`) ??
    resolveNestedValue(translations[locale], keys) ??
    resolveNestedValue(translations[locale], ["sealantern"].concat(keys))
  );
}

function interpolateVariables(template: string, options: Record<string, unknown>): string {
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

  setLocale(locale: string) {
    if (isSupportedLocale(locale) || pluginLocaleNames[locale] !== undefined) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    let resolved: string | undefined;
    for (const locale of getLocaleFallbackChain(this.currentLocale.value)) {
      resolved = resolveLocaleValue(locale, key);
      if (resolved !== undefined) {
        break;
      }
    }

    if (resolved === undefined) {
      for (const pluginMap of Object.values(pluginFlatTranslations)) {
        const currentLocaleValue = this.currentLocale.value;
        const val =
          pluginMap[currentLocaleValue]?.[key] ?? pluginMap["en-US"]?.[key] ?? pluginMap["zh-CN"]?.[key];
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
    for (const locale of getLocaleFallbackChain(this.currentLocale.value)) {
      if (resolveLocaleValue(locale, key) !== undefined) {
        return true;
      }
    }

    return Object.values(pluginFlatTranslations).some((pluginMap) => {
      const currentLocaleValue = this.currentLocale.value;
      return (
        pluginMap[currentLocaleValue]?.[key] !== undefined ||
        pluginMap["en-US"]?.[key] !== undefined ||
        pluginMap["zh-CN"]?.[key] !== undefined
      );
    });
  }

  getTranslations() {
    return translations as Record<string, LanguageFile>;
  }

  getLocaleMetadata(locale: string): LocaleMetadata | undefined {
    return builtInLocaleMetadata[locale] ?? (pluginLocaleNames[locale] ? { language: pluginLocaleNames[locale] } : undefined);
  }

  getLocaleDisplayName(locale: string): string | undefined {
    return this.getLocaleMetadata(locale)?.language;
  }

  getLocaleRef() {
    return this.currentLocale;
  }

  getAvailableLocales(): readonly LocaleCode[] {
    return supportedLocales;
  }

  isSupportedLocale(locale: string): boolean {
    return isSupportedLocale(locale) || pluginLocaleNames[locale] !== undefined;
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
