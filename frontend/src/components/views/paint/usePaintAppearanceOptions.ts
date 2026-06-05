import { computed, onMounted, ref } from "vue";
import { getSystemFonts, type WindowEffect } from "@api/settings";
import { i18n } from "@language";
import { usePluginStore } from "@stores/pluginStore";
import { isMacOSPlatform, isWindowsPlatform } from "@utils/platform";

export function usePaintAppearanceOptions() {
  const pluginStore = usePluginStore();
  const fontsLoading = ref(false);
  const isMacOS = isMacOSPlatform();
  const isWindows = isWindowsPlatform();

  const fontFamilyOptions = ref<{ label: string; value: string }[]>([
    { label: i18n.t("settings.font_family_default"), value: "" },
  ]);

  const themeProxyPlugin = computed(() => {
    return pluginStore.plugins.find(
      (plugin) =>
        plugin.state === "enabled" &&
        pluginStore.hasCapability(plugin.manifest.id, "theme-provider"),
    );
  });

  const isThemeProxied = computed(() => !!themeProxyPlugin.value);
  const themeProxyPluginName = computed(() => themeProxyPlugin.value?.manifest.name || "");

  const windowEffectOptions = computed<{ label: string; value: WindowEffect }[]>(() => {
    if (isMacOS) {
      return [
        { label: i18n.t("settings.window_effect_options.off"), value: "off" },
        { label: i18n.t("settings.window_effect_options.vibrancy"), value: "vibrancy" },
      ];
    }

    if (isWindows) {
      return [
        { label: i18n.t("settings.window_effect_options.off"), value: "off" },
        { label: i18n.t("settings.window_effect_options.auto"), value: "auto" },
        { label: i18n.t("settings.window_effect_options.mica"), value: "mica" },
        { label: i18n.t("settings.window_effect_options.acrylic"), value: "acrylic" },
        { label: i18n.t("settings.window_effect_options.blur"), value: "blur" },
      ];
    }

    return [{ label: i18n.t("settings.window_effect_options.off"), value: "off" }];
  });

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
    isThemeProxied,
    themeProxyPluginName,
    windowEffectOptions,
  };
}
