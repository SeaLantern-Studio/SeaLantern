import { computed, onMounted, ref } from "vue";
import { getSystemFonts } from "@api/settings";
import { i18n } from "@language";
import { useThemeProviderOwnership } from "@composables/useThemeProviderOwnership";

export function usePaintAppearanceOptions() {
  const fontsLoading = ref(false);
  const { isThemeProviderActive, themeProviderPluginName } = useThemeProviderOwnership();

  const fontFamilyOptions = ref<{ label: string; value: string }[]>([
    { label: i18n.t("settings.font_family_default"), value: "" },
  ]);

  const themeProviderNotice = computed(() => {
    if (!themeProviderPluginName.value) {
      return "";
    }

    return i18n.t("settings.paint.theme_provider_notice", {
      plugin: themeProviderPluginName.value,
    });
  });

  const memoryDisplayPrecisionOptions = computed<{ label: string; value: number }[]>(() => [
    { label: i18n.t("settings.memory_display_precision_options.0"), value: 0 },
    { label: i18n.t("settings.memory_display_precision_options.2"), value: 2 },
    { label: i18n.t("settings.memory_display_precision_options.4"), value: 4 },
  ]);

  async function loadSystemFonts() {
    fontsLoading.value = true;
    try {
      const fonts = await getSystemFonts();
      fontFamilyOptions.value = [
        { label: i18n.t("settings.font_family_default"), value: "" },
        ...fonts.map((font) => ({ label: font, value: `'${font}'` })),
      ];
    } catch (error) {
      console.error("Failed to load system fonts:", error);
    } finally {
      fontsLoading.value = false;
    }
  }

  onMounted(() => {
    void loadSystemFonts();
  });

  return {
    fontsLoading,
    fontFamilyOptions,
    isThemeProviderActive,
    themeProviderPluginName,
    themeProviderNotice,
    memoryDisplayPrecisionOptions,
  };
}
