import { getLocaleBundle } from "@api/i18n";
import { ensureLocaleLoaded, i18n, type LocaleCode } from "@language";

export async function fetchLocale(locale: LocaleCode) {
  try {
    const bundle = await getLocaleBundle(locale);
    return bundle.entries;
  } catch {
    await ensureLocaleLoaded(locale);
    const translations = i18n.getTranslations()[locale];
    if (!translations) {
      throw new Error(`Locale ${locale} not found in loaded translations`);
    }

    return {
      sealantern: translations,
    };
  }
}
