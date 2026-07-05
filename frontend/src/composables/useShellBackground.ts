import type { CSSProperties } from "vue";
import { computed } from "vue";
import type { WindowEffect } from "@api/settings";
import { useSettingsStore } from "@stores/settingsStore";
import { isLinuxPlatform, isMacOSPlatform, isWindowsPlatform } from "@utils/platform";

const WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED = true;

function resolveBackgroundSize(size: string): string {
  switch (size) {
    case "fill":
      return "100% 100%";
    case "contain":
    case "auto":
    case "cover":
      return size;
    default:
      return "cover";
  }
}

export function useShellBackground() {
  const settingsStore = useSettingsStore();
  const isWindows = isWindowsPlatform();
  const isMacOS = isMacOSPlatform();
  const isLinux = isLinuxPlatform();

  const backgroundImage = computed(() => settingsStore.backgroundImage || "");
  const backgroundOpacity = computed(() => settingsStore.backgroundOpacity ?? 0.3);
  const backgroundBlur = computed(() => settingsStore.backgroundBlur ?? 0);
  const backgroundBrightness = computed(() => settingsStore.backgroundBrightness ?? 1);
  const backgroundSize = computed(() => resolveBackgroundSize(settingsStore.backgroundSize || "cover"));
  const windowEffect = computed<WindowEffect>(
    () => (settingsStore.windowEffect || "off") as WindowEffect,
  );

  const hasBackgroundImage = computed(() => Boolean(backgroundImage.value));
  const supportsTransparentWindowBackdrop =
    isMacOS || (isWindows && !WINDOWS_NATIVE_WINDOW_EFFECTS_DISABLED);
  const backgroundStyle = computed<CSSProperties>(() => {
    if (!backgroundImage.value) {
      return {};
    }

    return {
      backgroundImage: `url(${backgroundImage.value})`,
      backgroundSize: backgroundSize.value,
      backgroundPosition: "center",
      backgroundRepeat: "no-repeat",
      opacity: backgroundOpacity.value,
      filter: `blur(${backgroundBlur.value}px) brightness(${backgroundBrightness.value})`,
    };
  });

  const hasTransparentWindowBackdrop = computed(
    () =>
      !isLinux &&
      supportsTransparentWindowBackdrop &&
      (windowEffect.value !== "off" || !backgroundImage.value),
  );

  return {
    hasBackgroundImage,
    backgroundStyle,
    hasTransparentWindowBackdrop,
  };
}
