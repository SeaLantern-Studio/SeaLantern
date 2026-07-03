import { ref, shallowRef, type Ref, type ShallowRef } from "vue";

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
type LocaleModuleLoader = () => Promise<LocaleModule>;

const DEFAULT_SYNC_LOCALE = "zh-CN";
const localeMetadataModules = import.meta.glob<LocaleMetadataModule>("./*/language.json", {
  eager: true,
});
const localeGroupModuleLoaders = import.meta.glob<LocaleModule>("./*/*.json");
const defaultLocaleModules = import.meta.glob<LocaleModule>("./zh-CN/*.json", {
  eager: true,
});
const builtInLocaleMetadata: Record<string, LocaleMetadata> = {};
const builtInLocaleGroups: Record<string, Record<string, TranslationValue>> = {};
const builtInLocaleGroupLoaders: Record<string, Record<string, LocaleModuleLoader>> = {};
const localeLoadPromises: Record<string, Promise<boolean> | undefined> = {};

for (const [path, module] of Object.entries(localeMetadataModules)) {
  const match = path.match(/^\.\/([^/]+)\/language\.json$/);
  if (!match) {
    continue;
  }

  const [, locale] = match;
  builtInLocaleMetadata[locale] = module.default;
}

for (const [path, loader] of Object.entries(localeGroupModuleLoaders)) {
  const match = path.match(/^\.\/([^/]+)\/([^/]+)\.json$/);
  if (!match) {
    continue;
  }

  const [, locale, group] = match;
  if (group === "language") {
    continue;
  }

  if (!builtInLocaleGroupLoaders[locale]) {
    builtInLocaleGroupLoaders[locale] = {};
  }
  builtInLocaleGroupLoaders[locale][group] = loader;
}

for (const [path, module] of Object.entries(defaultLocaleModules)) {
  const match = path.match(/^\.\/([^/]+)\/([^/]+)\.json$/);
  if (!match) {
    continue;
  }

  const [, locale, group] = match;
  if (group === "language") {
    continue;
  }

  if (!builtInLocaleGroups[locale]) {
    builtInLocaleGroups[locale] = {};
  }
  builtInLocaleGroups[locale][group] = normalizeGroupValue(module.default);
}

const translations: Record<string, LanguageFile> = {};
const supportedLocales: string[] = Array.from(
  new Set([
    ...Object.keys(builtInLocaleGroupLoaders),
    ...Object.keys(builtInLocaleGroups),
    ...Object.keys(builtInLocaleMetadata),
  ]),
).toSorted((a, b) => a.localeCompare(b));

const defaultLocaleTranslations = buildLocaleTranslations(DEFAULT_SYNC_LOCALE);
if (defaultLocaleTranslations) {
  translations[DEFAULT_SYNC_LOCALE] = defaultLocaleTranslations;
}

const backendFlatTranslations: Record<string, Record<string, string>> = {};

const pluginFlatTranslations: Record<string, Record<string, Record<string, string>>> = {};
const pluginLocaleNames: Record<string, string> = {};
const selectableLocales = new Set<string>(supportedLocales);
const localeRegistryVersion = shallowRef(0);
const currentLocaleRef = ref<LocaleCode>(DEFAULT_SYNC_LOCALE);
const DEFAULT_FALLBACK_LOCALES = ["en-US", DEFAULT_SYNC_LOCALE] as const;

export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

function markLocaleRegistryChanged(): void {
  localeRegistryVersion.value += 1;
}

function registerSupportedLocale(locale: string): boolean {
  if (supportedLocales.includes(locale)) {
    return false;
  }

  supportedLocales.push(locale);
  markLocaleRegistryChanged();
  return true;
}

function registerSelectableLocale(locale: string): void {
  const normalized = locale.trim();
  if (!normalized) {
    return;
  }

  registerSupportedLocale(normalized);
  if (selectableLocales.has(normalized)) {
    return;
  }

  selectableLocales.add(normalized);
  markLocaleRegistryChanged();
}

function dedupeLocaleCodes(locales: readonly string[]): string[] {
  return Array.from(new Set(locales.map((locale) => locale.trim()).filter(Boolean)));
}

function getRegisteredLocalesSorted(): LocaleCode[] {
  return [...supportedLocales].toSorted((left, right) => left.localeCompare(right));
}

function getSelectableLocalesSorted(): LocaleCode[] {
  return [...selectableLocales].toSorted((left, right) => left.localeCompare(right));
}

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  translations[locale] = normalizeLanguageFile(data);
  registerSelectableLocale(locale);
}

export async function ensureLocaleLoaded(locale: LocaleCode): Promise<boolean> {
  if (translations[locale]) {
    return true;
  }

  if (localeLoadPromises[locale]) {
    return localeLoadPromises[locale];
  }

  localeLoadPromises[locale] = loadBuiltInLocaleTranslations(locale)
    .then((localeTranslations) => {
      if (!localeTranslations) {
        return false;
      }

      translations[locale] = localeTranslations;
      registerSelectableLocale(locale);
      return true;
    })
    .finally(() => {
      delete localeLoadPromises[locale];
    });

  return localeLoadPromises[locale]!;
}

export function setLocaleBundle(
  locale: LocaleCode,
  entries: Record<string, string>,
  locales?: readonly string[],
) {
  backendFlatTranslations[locale] = entries;
  registerSelectableLocale(locale);
  if (locales) {
    for (const localeCode of locales) {
      registerSelectableLocale(localeCode);
    }
  }
}

export function registerPluginLocale(locale: string, displayName: string) {
  const displayNameChanged = pluginLocaleNames[locale] !== displayName;
  pluginLocaleNames[locale] = displayName;
  const localeAdded = registerSupportedLocale(locale);
  if (!localeAdded && displayNameChanged) {
    markLocaleRegistryChanged();
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

async function loadBuiltInLocaleTranslations(locale: string): Promise<LanguageFile | undefined> {
  const loaders = builtInLocaleGroupLoaders[locale];
  if (!loaders && !builtInLocaleGroups[locale]) {
    return undefined;
  }

  const localeGroups = builtInLocaleGroups[locale] ?? (builtInLocaleGroups[locale] = {});
  const missingGroups = Object.entries(loaders ?? {}).filter(([group]) => !(group in localeGroups));

  if (missingGroups.length > 0) {
    const loadedGroups = await Promise.all(
      missingGroups.map(async ([group, loader]) => {
        const module = await loader();
        return {
          group,
          value: normalizeGroupValue(module.default),
        };
      }),
    );

    for (const { group, value } of loadedGroups) {
      localeGroups[group] = value;
    }
  }

  return buildLocaleTranslations(locale);
}

function resolveNestedValue(
  source: TranslationNode | undefined,
  keys: string[],
): string | undefined {
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

function getLocaleFallbackChain(): LocaleCode[] {
  return dedupeLocaleCodes([currentLocaleRef.value, ...DEFAULT_FALLBACK_LOCALES]);
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
  private currentLocale: Ref<LocaleCode> = currentLocaleRef;

  setLocale(locale: string) {
    if (isSupportedLocale(locale) || pluginLocaleNames[locale] !== undefined) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    localeRegistryVersion.value;

    let resolved: string | undefined;
    for (const locale of getLocaleFallbackChain()) {
      resolved = resolveLocaleValue(locale, key);
      if (resolved !== undefined) {
        break;
      }
    }

    if (resolved === undefined) {
      for (const pluginMap of Object.values(pluginFlatTranslations)) {
        for (const locale of getLocaleFallbackChain()) {
          const val = pluginMap[locale]?.[key];
          if (val !== undefined) {
            resolved = val;
            break;
          }
        }

        if (resolved !== undefined) {
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
    localeRegistryVersion.value;

    for (const locale of getLocaleFallbackChain()) {
      if (resolveLocaleValue(locale, key) !== undefined) {
        return true;
      }
    }

    return Object.values(pluginFlatTranslations).some((pluginMap) => {
      return getLocaleFallbackChain().some((locale) => pluginMap[locale]?.[key] !== undefined);
    });
  }

  getTranslations() {
    return translations as Record<string, LanguageFile>;
  }

  getLocaleMetadata(locale: string): LocaleMetadata | undefined {
    localeRegistryVersion.value;

    return (
      builtInLocaleMetadata[locale] ??
      (pluginLocaleNames[locale] ? { language: pluginLocaleNames[locale] } : undefined)
    );
  }

  getLocaleDisplayName(locale: string): string | undefined {
    return this.getLocaleMetadata(locale)?.language;
  }

  getLocaleRef() {
    return this.currentLocale;
  }

  getAvailableLocales(): readonly LocaleCode[] {
    localeRegistryVersion.value;
    return getSelectableLocalesSorted();
  }

  getRegisteredLocales(): readonly LocaleCode[] {
    localeRegistryVersion.value;
    return getRegisteredLocalesSorted();
  }

  getLocaleRegistryVersionRef(): ShallowRef<number> {
    return localeRegistryVersion;
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
