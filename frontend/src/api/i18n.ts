import { tauriInvoke } from "@api/tauri";

export interface LocaleBundleResponse {
  locale: string;
  entries: Record<string, string>;
  available_locales: string[];
}

export async function getLocaleBundle(locale?: string): Promise<LocaleBundleResponse> {
  return tauriInvoke("get_locale_bundle", locale ? { locale } : undefined);
}
