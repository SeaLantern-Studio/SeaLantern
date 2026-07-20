import { ref, type Ref } from "vue";

// 动态导入所有语言文件 ,此处不用别名导入，因为需要使用 import.meta.glob 来获取文件路径
const languageFiles: Record<string, any> = import.meta.glob("./*.json", { eager: true });

// 处理语言文件，提取语言代码和数据
const processLanguageFiles = () => {
  const translations: Record<string, LanguageFile> = {};
  const supportedLocales: string[] = [];

  // 遍历所有导入的语言文件
  for (const [path, module] of Object.entries(languageFiles)) {
    // 从文件路径中提取语言代码，如 "./zh-CN.json" -> "zh-CN"
    const match = path.match(/\.\/(.*)\.json$/);
    if (match) {
      const localeCode = match[1];
      const data = (module as any).default;

      // 确保数据是有效的语言文件
      if (data && typeof data === "object") {
        translations[localeCode] = data;
        supportedLocales.push(localeCode);
      }
    }
  }

  return { translations, supportedLocales };
};

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

// 语言文件类型，包含语言名称字段
type LanguageFile = TranslationNode & {
  languageName?: string;
};

const { translations, supportedLocales } = processLanguageFiles();

const pluginFlatTranslations: Record<string, Record<string, Record<string, string>>> = {};
const pluginLocaleNames: Record<string, string> = {};

export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  if (isSupportedLocale(locale)) {
    translations[locale] = data;
  }
  i18n.invalidateCache();
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
  i18n.invalidateCache();
}

export function removePluginTranslations(pluginId: string) {
  delete pluginFlatTranslations[pluginId];
  i18n.invalidateCache();
}

export function getPluginLocaleDisplayName(locale: string): string | undefined {
  return pluginLocaleNames[locale];
}

function isSupportedLocale(locale: string): locale is LocaleCode {
  return supportedLocales.includes(locale);
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
  // 翻译原文缓存: locale -> (dottedKey -> 原文)
  // 命中后跳过 4 次嵌套查找与 pluginFlatTranslations 遍历
  private translationCache: Map<LocaleCode, Map<string, string>> = new Map();
  private cacheValidLocales: Set<LocaleCode> = new Set();

  /** 失效所有缓存(在 translations / plugin 翻译变更时调用) */
  invalidateCache(): void {
    this.translationCache.clear();
    this.cacheValidLocales.clear();
  }

  /** 确保 locale 对应的扁平化缓存已构建 */
  private ensureCacheForLocale(locale: LocaleCode): Map<string, string> | null {
    if (this.cacheValidLocales.has(locale)) {
      return this.translationCache.get(locale) || null;
    }
    const data = translations[locale];
    if (!data) return null;
    const flat = new Map<string, string>();

    // 递归展开节点,写入 flat(已存在的 key 不覆盖,以此实现优先级)
    const walkInto = (node: TranslationNode, prefix: string[]) => {
      for (const [k, v] of Object.entries(node)) {
        if (v && typeof v === "object") {
          walkInto(v, [...prefix, k]);
        } else if (typeof v === "string") {
          const dotted = [...prefix, k].join(".");
          if (!flat.has(dotted)) flat.set(dotted, v);
        }
      }
    };

    // 优先级由高到低:
    // 1) sealantern 子树的无前缀 key (对应原 t() 先查 data.sealantern.foo)
    const sealanternRoot = (data as any).sealantern;
    if (sealanternRoot && typeof sealanternRoot === "object") {
      walkInto(sealanternRoot as TranslationNode, []);
    }
    // 2) data 根的全路径 key (含 "sealantern.xxx" 与根级 string 如 "languageName")
    walkInto(data, []);
    // 3) 插件翻译(优先级最低,仅补缺)
    for (const pluginMap of Object.values(pluginFlatTranslations)) {
      const entries = pluginMap[locale];
      if (entries) {
        for (const [k, v] of Object.entries(entries)) {
          if (!flat.has(k)) flat.set(k, v);
        }
      }
    }

    this.translationCache.set(locale, flat);
    this.cacheValidLocales.add(locale);
    return flat;
  }

  setLocale(locale: string) {
    if (isSupportedLocale(locale) || pluginLocaleNames[locale] !== undefined) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    const currentLocaleValue = this.currentLocale.value;

    // 优先命中当前 locale 缓存
    let resolved: string | undefined;
    const currentCache = this.ensureCacheForLocale(currentLocaleValue);
    if (currentCache) {
      resolved = currentCache.get(key);
    }
    // 回退到 fallback locale
    if (resolved === undefined && currentLocaleValue !== this.fallbackLocale) {
      const fallbackCache = this.ensureCacheForLocale(this.fallbackLocale);
      if (fallbackCache) {
        resolved = fallbackCache.get(key);
      }
    }

    if (resolved === undefined) {
      return key;
    }

    if (Object.keys(options).length === 0) return resolved;
    return interpolateVariables(resolved, options);
  }

  te(key: string): boolean {
    const currentLocaleValue = this.currentLocale.value;
    const currentCache = this.ensureCacheForLocale(currentLocaleValue);
    if (currentCache?.has(key)) return true;
    if (currentLocaleValue !== this.fallbackLocale) {
      const fallbackCache = this.ensureCacheForLocale(this.fallbackLocale);
      if (fallbackCache?.has(key)) return true;
    }
    return false;
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
  registerPluginLocale,
  addPluginTranslations,
  removePluginTranslations,
  getPluginLocaleDisplayName,
};

export default languageAPI;
