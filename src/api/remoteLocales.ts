import { SUPPORTED_LOCALES, type LocaleCode } from "../locales";

const REMOTE_BASE = (import.meta as any).env?.VITE_LOCALES_BASE || "";

export type ProgressCallback = (locale: LocaleCode, loaded: number, total: number | null) => void;

async function fetchJsonWithProgress(url: string, onProgress?: (loaded: number, total: number | null) => void) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch ${url}: ${res.status}`);

  const contentLength = res.headers.get("content-length");
  const total = contentLength ? parseInt(contentLength, 10) : null;

  if (!res.body || !onProgress) {
    return res.json();
  }

  const reader = res.body.getReader();
  const chunks: Uint8Array[] = [];
  let received = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    if (value) {
      chunks.push(value);
      received += value.length;
      onProgress(received, total);
    }
  }

  const full = new TextDecoder("utf-8").decode(concatUint8(chunks));
  return JSON.parse(full);
}

function concatUint8(chunks: Uint8Array[]) {
  const total = chunks.reduce((s, c) => s + c.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.length;
  }
  return out;
}

export async function fetchLocale(
  locale: LocaleCode,
  onProgress?: (loaded: number, total: number | null) => void,
  baseUrl?: string,
) {
  const base = (baseUrl || REMOTE_BASE).replace(/\/$/, "");
  if (!base) throw new Error("REMOTE_LOCALES base URL not configured");
  const url = `${base}/${locale}.json`;
  return fetchJsonWithProgress(url, onProgress);
}

export async function fetchByUrl(url: string, onProgress?: (loaded: number, total: number | null) => void) {
  return fetchJsonWithProgress(url, onProgress);
}

export async function downloadAllLocales(onProgress?: ProgressCallback, baseUrl?: string) {
  const results: Record<string, unknown> = {};
  for (const locale of SUPPORTED_LOCALES) {
    try {
      const partialProgress = (loaded: number, total: number | null) => {
        onProgress?.(locale as LocaleCode, loaded, total);
      };
      const data = await fetchLocale(locale as any, partialProgress, baseUrl);
      results[locale] = data;
    } catch (e) {
      // ignore single-locale failures, continue
      console.warn(`Failed to download locale ${locale}:`, e);
    }
  }
  return results as Record<LocaleCode, Record<string, unknown>>;
}
