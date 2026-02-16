import { computed, onMounted } from "vue";
import { defineStore } from "pinia";
import { i18n, type LocaleCode } from "../locales";
import { settingsApi } from "../api/settings";

const LOCALE_LABEL_KEYS: Record<LocaleCode, string> = {
  "zh-CN": "header.chinese",
  "zh-TW": "header.chinese_tw",
  "en-US": "header.english",
};

export const useI18nStore = defineStore("i18n", () => {
  const localeRef = i18n.getLocaleRef();
  const supportedLocales = i18n.getAvailableLocales();

  const locale = computed(() => localeRef.value);
  const currentLocale = computed(() => localeRef.value);
  const isChinese = computed(
    () => localeRef.value === "zh-CN" || localeRef.value === "zh-TW"
  );
  const isSimplifiedChinese = computed(() => localeRef.value === "zh-CN");
  const isTraditionalChinese = computed(() => localeRef.value === "zh-TW");
  const isEnglish = computed(() => localeRef.value === "en-US");
  const localeOptions = computed(() =>
    supportedLocales.map((code) => ({
      code,
      labelKey: LOCALE_LABEL_KEYS[code],
    }))
  );

  async function setLocale(nextLocale: string) {
    if (i18n.isSupportedLocale(nextLocale)) {
      i18n.setLocale(nextLocale);
      // 保存语言设置到持久化存储
      try {
        const settings = await settingsApi.get();
        settings.language = nextLocale;
        await settingsApi.save(settings);
      } catch (error) {
        console.error("Failed to save language setting:", error);
      }
    }
  }

  function toggleLocale() {
    const currentIndex = supportedLocales.indexOf(localeRef.value);
    const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % supportedLocales.length;
    setLocale(supportedLocales[nextIndex]);
  }

  // 从持久化存储加载语言设置
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

  // 组件挂载时加载语言设置
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
  };
});
