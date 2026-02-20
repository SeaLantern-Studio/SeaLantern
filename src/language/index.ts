import { ref, type Ref } from "vue";

// 动态导入所有语言文件
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
      if (data && typeof data === 'object') {
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

// 处理语言文件
const { translations, supportedLocales } = processLanguageFiles();

// 导出支持的语言列表
export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  if (isSupportedLocale(locale)) {
    translations[locale] = data;
  }
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
    return translations as Record<string, LanguageFile>;
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
