import { getLocaleBundle } from "@api/i18n";
import { setLocaleBundle } from "@language";
import { normalizeAppError } from "@utils/appError";

export async function loadLocaleBundle(locale: string): Promise<void> {
  const bundle = await getLocaleBundle(locale);
  setLocaleBundle(bundle.locale, bundle.entries, bundle.available_locales);
}

export async function tryLoadLocaleBundle(locale: string): Promise<boolean> {
  try {
    await loadLocaleBundle(locale);
    return true;
  } catch (error) {
    console.error("Failed to load locale bundle:", normalizeAppError(error));
    return false;
  }
}
