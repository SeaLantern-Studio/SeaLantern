import { i18n, type LocaleCode } from "@language";

// 从已加载的翻译数据中获取翻译
export async function fetchLocale(locale: LocaleCode) {
  const translations = i18n.getTranslations();
  const data = translations[locale];
  if (!data) {
    throw new Error(`Locale ${locale} not found in loaded translations`);
  }
  return data;
}
