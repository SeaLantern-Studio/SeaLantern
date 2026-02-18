import { computed, onMounted, reactive } from "vue";
import { defineStore } from "pinia";
import { i18n, type LocaleCode, setTranslations } from "../locales";
import { settingsApi } from "../api/settings";
import { fetchLocale, fetchByUrl } from "../api/remoteLocales";
import { REMOTE_LOCALES_MAP } from "./i18nRemote";

const LOCALE_LABEL_KEYS: Record<string, string> = {
  "zh-CN": "header.chinese",
  "en-US": "header.english",
  "zh-TW": "header.chinese_tw",
  "zh-JB": "header.chinese_jb",
  "zh-NE": "header.chinese_dongbei",
  "de-DE": "header.deutsch",
  "en-AU": "header.aussie",
  "en-GB": "header.british",
  "en-PT": "header.pirate",
  "en-UN": "header.upsidedown",
  "es-ES": "header.spanish",
  "ja-JP": "header.japanese",
  "ru-RU": "header.russian",
  "vi-VN": "header.vietnamese",
  "zh-CT": "header.cantonese",
  "zh-CY": "header.chinese_cy",
  "zh-HN": "header.chinese_hn",
  "zh-JL": "header.chinese_jl",
  "zh-ME": "header.chinese_meow",
  "zh-MN": "header.chinese_hokkien",
  "zh-TJ": "header.chinese_tj",
  "zh-WU": "header.chinese_wu",
  "ja-KS": "header.kansaiben",
  "ja-HK": "header.hokkaidou",
  "ko-KR": "header.korean",
  "ko-NK": "header.north_korean",
  "fr-FA": "header.french",
  "fr-CA": "header.french_ca",
  "es-AR": "header.spanish_ar",
  "zh-HX": "header.huoxing"
};

export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const supportedLocales = i18n.getAvailableLocales();

  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(() => localeRef.value === "zh-CN" || localeRef.value === "zh-TW");
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");
  const localeOptions = computed(() =>
    supportedLocales.map((code) => ({
      code,
      labelKey: LOCALE_LABEL_KEYS[code],
    })),
  );

  // 下载进度：{ [locale]: { loaded, total } }
  const downloadProgress = reactive<Record<string, { loaded: number; total: number | null }>>({});
  // per-locale clear timers: 清除 100% 显示的定时器
  const clearTimers: Record<string, number> = {};

  function getLocaleProgress(code: LocaleCode) {
    const p = downloadProgress[code];
    if (!p) return 0;
    if (!p.total) return p.loaded > 0 ? 50 : 0;
    return Math.min(100, Math.round((p.loaded / p.total) * 100));
  }

  async function setLocale(nextLocale: string) {
    if (i18n.isSupportedLocale(nextLocale)) {
      i18n.setLocale(nextLocale);
      try {
        const settings = await settingsApi.get();
        settings.language = nextLocale;
        await settingsApi.save(settings);
      } catch (error) {
        console.error("Failed to save language setting:", error);
      }
    }
  }

  async function downloadLocale(localeCode: string) {
    if (!i18n.isSupportedLocale(localeCode)) return;
    try {
      const s = await settingsApi.get();
      const base = (s as any).locales_base_url || (import.meta as any).env?.VITE_LOCALES_BASE || "";

      const onProgress = (loaded: number, total: number | null) => {
        downloadProgress[localeCode] = { loaded, total };
        if (total && loaded >= total) {
          if (clearTimers[localeCode]) {
            clearTimeout(clearTimers[localeCode]);
          }
          clearTimers[localeCode] = window.setTimeout(() => {
            try { delete downloadProgress[localeCode]; } catch (e) {}
            delete clearTimers[localeCode];
          }, 3000);
        }
      };
      const mapped = (REMOTE_LOCALES_MAP as any)[localeCode];
      let data: any = null;
      if (mapped) {
        data = await fetchByUrl(mapped, onProgress);
      } else {
        data = await fetchLocale(localeCode as LocaleCode, onProgress, base || undefined);
      }
      setTranslations(localeCode as any, data as any);
      const p = downloadProgress[localeCode];
      if (!p || (p.total && p.loaded >= (p.total ?? 0))) {
        if (!p || p.total === null) {
          downloadProgress[localeCode] = { loaded: 1, total: 1 };
        }
        if (clearTimers[localeCode]) {
          clearTimeout(clearTimers[localeCode]);
        }
        clearTimers[localeCode] = window.setTimeout(() => {
          try { delete downloadProgress[localeCode]; } catch (e) {}
          delete clearTimers[localeCode];
        }, 3000);
      }
    } catch (e) {
      console.error("Failed to download locale:", localeCode, e);
    }
  }

  function toggleLocale() {
    const currentIndex = supportedLocales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % supportedLocales.length;
    setLocale(supportedLocales[nextIndex]);
  }

  async function loadLanguageSetting() {
    try {
      const settings = await settingsApi.get();
      if (settings.language && i18n.isSupportedLocale(settings.language)) {
        i18n.setLocale(settings.language);
      }
    } catch (error) {
      console.error("Failed to load language setting:", error);
    }
  }

  onMounted(() => {
    loadLanguageSetting();
  });

  return {
    locale,
    currentLocale,
    isChinese,
    isSimplifiedChinese,
    isTraditionalChinese,
    isEnglish,
    localeOptions,
    setLocale,
    toggleLocale,
    loadLanguageSetting,
    downloadLocale,
    getLocaleProgress,
    downloadProgress,
  };
});
