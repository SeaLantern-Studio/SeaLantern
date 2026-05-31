import { getLocaleBundle } from "@api/i18n";
import { i18n, type LocaleCode } from "@language";

export async function fetchLocale(locale: LocaleCode) {
  try {
    const bundle = await getLocaleBundle(locale);
    return bundle.entries;
  } catch {
    const translations = i18n.getTranslations();
    const data = translations[locale];
    if (!data) {
      throw new Error(`Locale ${locale} not found in loaded translations`);
    }
    return data;
  }
}
