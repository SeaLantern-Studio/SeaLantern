import { computed, onMounted, shallowRef, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AppSettings, PartialSettings, WindowEffect } from "@api/settings";
import { getSystemFonts } from "@api/settings";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useThemeProviderOwnership } from "@composables/useThemeProviderOwnership";
import { useToast } from "@composables/useToast";
import { useSettingsStore } from "@stores/settingsStore";
import { isMacOSPlatform, isWindowsPlatform } from "@utils/platform";

export interface AppearanceSelectOption<T extends string | number> {
  label: string;
  value: T;
  subLabel?: string;
}

type SliderFieldKey = "fontSize" | "backgroundOpacity" | "backgroundBlur" | "backgroundBrightness";

interface PendingSliderSave {
  partial: PartialSettings;
  failureMessage: string;
}

const FONT_SIZE_MIN = 12;
const FONT_SIZE_MAX = 24;
const FONT_SIZE_STEP = 1;
const BACKGROUND_OPACITY_MIN = 0;
const BACKGROUND_OPACITY_MAX = 1;
const BACKGROUND_OPACITY_STEP = 0.05;
const BACKGROUND_BLUR_MIN = 0;
const BACKGROUND_BLUR_MAX = 20;
const BACKGROUND_BLUR_STEP = 1;
const BACKGROUND_BRIGHTNESS_MIN = 0;
const BACKGROUND_BRIGHTNESS_MAX = 2;
const BACKGROUND_BRIGHTNESS_STEP = 0.1;
const SLIDER_DEBOUNCE_MS = 220;

function clampNumber(value: number, min: number, max: number): number {
  if (Number.isNaN(value)) {
    return min;
  }
  return Math.min(Math.max(value, min), max);
}

function roundToStep(value: number, step: number): number {
  if (step <= 0) {
    return value;
  }
  return Math.round(value / step) * step;
}

function normalizeNumber(value: number, min: number, max: number, step: number): number {
  const rounded = roundToStep(value, step);
  const clamped = clampNumber(rounded, min, max);
  return Number(clamped.toFixed(2));
}

function extractFileName(path: string): string {
  if (!path) {
    return "";
  }

  const segments = path.split(/[/\\]/);
  return segments.at(-1) || path;
}

function formatFontLabel(fontFamily: string): string {
  return fontFamily.replace(/^'+|'+$/g, "") || fontFamily;
}

export function useAppearanceSettingsSection() {
  const settingsStore = useSettingsStore();
  const toast = useToast();
  const { isThemeProviderActive, themeProviderPluginName } = useThemeProviderOwnership();

  const fontsLoading = shallowRef(false);
  const backgroundExpanded = shallowRef(Boolean(settingsStore.settings.background_image));

  const fontSizeDraft = shallowRef(14);
  const backgroundOpacityDraft = shallowRef(0.3);
  const backgroundBlurDraft = shallowRef(0);
  const backgroundBrightnessDraft = shallowRef(1);

  const fontFamilyOptions = shallowRef<AppearanceSelectOption<string>[]>([
    { label: i18n.t("settings.font_family_default"), value: "" },
  ]);

  const pendingSliderSaves = new Map<SliderFieldKey, PendingSliderSave>();
  const sliderTimers = new Map<SliderFieldKey, ReturnType<typeof setTimeout>>();

  let saveQueue: Promise<void> = Promise.resolve();

  function applyLocalSettings(partial: PartialSettings, applyClientSettings = false): void {
    settingsStore.updateSettings(partial as Partial<AppSettings>);

    if (applyClientSettings) {
      void settingsStore.queueClientSettingsApply();
    }
  }

  async function restorePersistedSettings(): Promise<void> {
    await settingsStore.loadSettings(true);
    void settingsStore.queueClientSettingsApply();
  }

  function enqueueSave(task: () => Promise<void>): Promise<void> {
    saveQueue = saveQueue.then(task, task);
    return saveQueue;
  }

  async function persistPartial(
    partial: PartialSettings,
    failureMessage: string,
  ): Promise<boolean> {
    try {
      await enqueueSave(async () => {
        await settingsStore.updatePartial(partial);
      });
      return true;
    } catch (error) {
      console.error("Failed to save appearance settings.", error);
      toast.error(failureMessage);
      await restorePersistedSettings();
      return false;
    }
  }

  function clearSliderTimer(key: SliderFieldKey): void {
    const timer = sliderTimers.get(key);
    if (timer) {
      clearTimeout(timer);
      sliderTimers.delete(key);
    }
  }

  function scheduleSliderSave(
    key: SliderFieldKey,
    partial: PartialSettings,
    failureMessage: string,
  ): void {
    pendingSliderSaves.set(key, { partial, failureMessage });
    clearSliderTimer(key);

    const timer = setTimeout(() => {
      void flushSliderSave(key);
    }, SLIDER_DEBOUNCE_MS);

    sliderTimers.set(key, timer);
  }

  async function flushSliderSave(key: SliderFieldKey): Promise<void> {
    clearSliderTimer(key);
    const pending = pendingSliderSaves.get(key);
    if (!pending) {
      return;
    }

    pendingSliderSaves.delete(key);
    await persistPartial(pending.partial, pending.failureMessage);
  }

  async function loadFontOptions(): Promise<void> {
    fontsLoading.value = true;
    try {
      const fonts = await getSystemFonts();
      const options: AppearanceSelectOption<string>[] = [
        { label: i18n.t("settings.font_family_default"), value: "" },
        ...fonts.map((font) => ({
          label: font,
          value: `'${font}'`,
          subLabel: font,
        })),
      ];

      const currentFontFamily = settingsStore.settings.font_family || "";
      if (currentFontFamily && !options.some((option) => option.value === currentFontFamily)) {
        options.splice(1, 0, {
          label: formatFontLabel(currentFontFamily),
          value: currentFontFamily,
          subLabel: currentFontFamily,
        });
      }

      fontFamilyOptions.value = options;
    } catch (error) {
      console.error("Failed to load system fonts.", error);
      fontFamilyOptions.value = [{ label: i18n.t("settings.font_family_default"), value: "" }];
    } finally {
      fontsLoading.value = false;
    }
  }

  const isWindows = isWindowsPlatform();
  const isMacOS = isMacOSPlatform();

  const themeProviderNotice = computed(() => {
    if (!themeProviderPluginName.value) {
      return "";
    }
    return i18n.t("settings.paint.theme_provider_notice", {
      plugin: themeProviderPluginName.value,
    });
  });

  const isReady = computed(() => settingsStore.isLoaded);
  const hasBackgroundImage = computed(() => Boolean(settingsStore.settings.background_image));
  const backgroundImagePath = computed(() => settingsStore.settings.background_image || "");
  const backgroundPreviewUrl = computed(() => {
    if (!backgroundImagePath.value) {
      return "";
    }
    return convertFileSrc(backgroundImagePath.value);
  });
  const backgroundImageName = computed(() => extractFileName(backgroundImagePath.value));

  const theme = computed(() => settingsStore.settings.theme || "auto");
  const fontFamily = computed(() => settingsStore.settings.font_family || "");
  const windowEffect = computed<WindowEffect>(() => {
    return (settingsStore.settings.window_effect || "off") as WindowEffect;
  });
  const minimalMode = computed(() => Boolean(settingsStore.settings.minimal_mode));
  const backgroundSize = computed(() => settingsStore.settings.background_size || "cover");

  const themeOptions = computed<AppearanceSelectOption<string>[]>(() => [
    { label: i18n.t("settings.theme_options.auto"), value: "auto" },
    { label: i18n.t("settings.theme_options.light"), value: "light" },
    { label: i18n.t("settings.theme_options.dark"), value: "dark" },
  ]);

  const windowEffectOptions = computed<AppearanceSelectOption<WindowEffect>[]>(() => {
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

  const backgroundSizeOptions = computed<AppearanceSelectOption<string>[]>(() => [
    { label: i18n.t("settings.background_size_options.cover"), value: "cover" },
    { label: i18n.t("settings.background_size_options.contain"), value: "contain" },
    { label: i18n.t("settings.background_size_options.fill"), value: "fill" },
    { label: i18n.t("settings.background_size_options.auto"), value: "auto" },
  ]);

  const windowEffectHint = computed(() => {
    if (windowEffectOptions.value.length === 1 && windowEffectOptions.value[0]?.value === "off") {
      return i18n.t("settings.next.appearance.window_effect_fallback");
    }
    return "";
  });

  async function setTheme(value: string): Promise<void> {
    if (value === theme.value) {
      return;
    }

    applyLocalSettings({ theme: value }, true);
    await persistPartial({ theme: value }, i18n.t("settings.next.appearance.save_theme_failed"));
  }

  async function setFontFamily(value: string): Promise<void> {
    if (value === fontFamily.value) {
      return;
    }

    applyLocalSettings({ font_family: value }, true);
    await persistPartial(
      { font_family: value },
      i18n.t("settings.next.appearance.save_font_failed"),
    );
  }

  async function setWindowEffect(value: WindowEffect): Promise<void> {
    if (value === windowEffect.value) {
      return;
    }

    applyLocalSettings({ window_effect: value }, true);
    await persistPartial(
      { window_effect: value },
      i18n.t("settings.next.appearance.save_window_effect_failed"),
    );
  }

  async function setMinimalMode(value: boolean): Promise<void> {
    if (value === minimalMode.value) {
      return;
    }

    applyLocalSettings({ minimal_mode: value }, true);
    await persistPartial(
      { minimal_mode: value },
      i18n.t("settings.next.appearance.save_minimal_mode_failed"),
    );
  }

  async function setBackgroundSize(value: string): Promise<void> {
    if (value === backgroundSize.value) {
      return;
    }

    applyLocalSettings({ background_size: value });
    await persistPartial(
      { background_size: value },
      i18n.t("settings.next.appearance.save_background_failed"),
    );
  }

  function setFontSize(value: number): void {
    const nextValue = normalizeNumber(value, FONT_SIZE_MIN, FONT_SIZE_MAX, FONT_SIZE_STEP);
    fontSizeDraft.value = nextValue;
    applyLocalSettings({ font_size: nextValue }, true);
    scheduleSliderSave(
      "fontSize",
      { font_size: nextValue },
      i18n.t("settings.next.appearance.save_font_size_failed"),
    );
  }

  function commitFontSize(): void {
    void flushSliderSave("fontSize");
  }

  function setBackgroundOpacity(value: number): void {
    const nextValue = normalizeNumber(
      value,
      BACKGROUND_OPACITY_MIN,
      BACKGROUND_OPACITY_MAX,
      BACKGROUND_OPACITY_STEP,
    );
    backgroundOpacityDraft.value = nextValue;
    applyLocalSettings({ background_opacity: nextValue });
    scheduleSliderSave(
      "backgroundOpacity",
      { background_opacity: nextValue },
      i18n.t("settings.next.appearance.save_background_failed"),
    );
  }

  function commitBackgroundOpacity(): void {
    void flushSliderSave("backgroundOpacity");
  }

  function setBackgroundBlur(value: number): void {
    const nextValue = normalizeNumber(
      value,
      BACKGROUND_BLUR_MIN,
      BACKGROUND_BLUR_MAX,
      BACKGROUND_BLUR_STEP,
    );
    backgroundBlurDraft.value = nextValue;
    applyLocalSettings({ background_blur: nextValue });
    scheduleSliderSave(
      "backgroundBlur",
      { background_blur: nextValue },
      i18n.t("settings.next.appearance.save_background_failed"),
    );
  }

  function commitBackgroundBlur(): void {
    void flushSliderSave("backgroundBlur");
  }

  function setBackgroundBrightness(value: number): void {
    const nextValue = normalizeNumber(
      value,
      BACKGROUND_BRIGHTNESS_MIN,
      BACKGROUND_BRIGHTNESS_MAX,
      BACKGROUND_BRIGHTNESS_STEP,
    );
    backgroundBrightnessDraft.value = nextValue;
    applyLocalSettings({ background_brightness: nextValue });
    scheduleSliderSave(
      "backgroundBrightness",
      { background_brightness: nextValue },
      i18n.t("settings.next.appearance.save_background_failed"),
    );
  }

  function commitBackgroundBrightness(): void {
    void flushSliderSave("backgroundBrightness");
  }

  async function pickBackgroundImage(): Promise<void> {
    try {
      const selectedPath = await systemApi.pickImageFile();
      if (!selectedPath) {
        return;
      }

      backgroundExpanded.value = true;
      applyLocalSettings({ background_image: selectedPath }, true);
      await persistPartial(
        { background_image: selectedPath },
        i18n.t("settings.next.appearance.save_background_image_failed"),
      );
    } catch (error) {
      console.error("Failed to pick background image.", error);
      toast.error(i18n.t("settings.next.appearance.pick_image_failed"));
    }
  }

  async function removeBackgroundImage(): Promise<void> {
    if (!backgroundImagePath.value) {
      return;
    }

    applyLocalSettings({ background_image: "" }, true);
    await persistPartial(
      { background_image: "" },
      i18n.t("settings.next.appearance.save_background_image_failed"),
    );
  }

  function setBackgroundExpanded(value: boolean): void {
    backgroundExpanded.value = value;
  }

  watch(
    () => settingsStore.settings.font_size,
    (value) => {
      fontSizeDraft.value = normalizeNumber(
        value || 14,
        FONT_SIZE_MIN,
        FONT_SIZE_MAX,
        FONT_SIZE_STEP,
      );
    },
    { immediate: true },
  );

  watch(
    () => settingsStore.settings.background_opacity,
    (value) => {
      backgroundOpacityDraft.value = normalizeNumber(
        value ?? 0.3,
        BACKGROUND_OPACITY_MIN,
        BACKGROUND_OPACITY_MAX,
        BACKGROUND_OPACITY_STEP,
      );
    },
    { immediate: true },
  );

  watch(
    () => settingsStore.settings.background_blur,
    (value) => {
      backgroundBlurDraft.value = normalizeNumber(
        value ?? 0,
        BACKGROUND_BLUR_MIN,
        BACKGROUND_BLUR_MAX,
        BACKGROUND_BLUR_STEP,
      );
    },
    { immediate: true },
  );

  watch(
    () => settingsStore.settings.background_brightness,
    (value) => {
      backgroundBrightnessDraft.value = normalizeNumber(
        value ?? 1,
        BACKGROUND_BRIGHTNESS_MIN,
        BACKGROUND_BRIGHTNESS_MAX,
        BACKGROUND_BRIGHTNESS_STEP,
      );
    },
    { immediate: true },
  );

  watch(
    () => settingsStore.settings.background_image,
    (value, previousValue) => {
      if (!previousValue && value) {
        backgroundExpanded.value = true;
      }
    },
  );

  onMounted(() => {
    void loadFontOptions();
  });

  return {
    isReady,
    fontsLoading,
    theme,
    fontSize: fontSizeDraft,
    fontFamily,
    windowEffect,
    minimalMode,
    backgroundSize,
    backgroundOpacity: backgroundOpacityDraft,
    backgroundBlur: backgroundBlurDraft,
    backgroundBrightness: backgroundBrightnessDraft,
    backgroundExpanded,
    hasBackgroundImage,
    backgroundImagePath,
    backgroundPreviewUrl,
    backgroundImageName,
    isThemeProviderActive,
    themeProviderNotice,
    windowEffectHint,
    themeOptions,
    fontFamilyOptions,
    windowEffectOptions,
    backgroundSizeOptions,
    setTheme,
    setFontSize,
    commitFontSize,
    setFontFamily,
    setWindowEffect,
    setMinimalMode,
    setBackgroundSize,
    setBackgroundOpacity,
    commitBackgroundOpacity,
    setBackgroundBlur,
    commitBackgroundBlur,
    setBackgroundBrightness,
    commitBackgroundBrightness,
    pickBackgroundImage,
    removeBackgroundImage,
    setBackgroundExpanded,
  };
}
