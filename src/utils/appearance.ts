import { desktopApi, type WindowMaterial, type WindowTheme } from "@api/desktop";
import type { AppSettings } from "@api/settings";
import { applyAcrylicEffect } from "@utils/acrylic";
import { isMacOSPlatform, isWindowsPlatform } from "@utils/platform";
import {
  applyColors,
  applyFontFamily,
  applyFontSize,
  applyTheme,
  isThemeProviderActive,
} from "@utils/theme";

const isMacOS = isMacOSPlatform();
const isWindows = isWindowsPlatform();

let lastNativeAppearance = "";
let appearanceApplyQueue: Promise<void> = Promise.resolve();

function getNativeMaterial(enabled: boolean): WindowMaterial {
  if (!enabled) return "solid";
  if (isMacOS) return "liquid_glass";
  if (isWindows) return "acrylic";
  return "solid";
}

function getWindowTheme(theme: string): WindowTheme {
  return theme === "light" || theme === "dark" ? theme : "auto";
}

async function applyAppearance(settings: AppSettings): Promise<void> {
  const root = document.documentElement;
  const nativeMaterialEnabled = settings.acrylic_enabled;
  const acrylicEnabled = root.getAttribute("data-acrylic") === "on";
  const enablingMaterial = !acrylicEnabled && nativeMaterialEnabled;
  const effectiveTheme =
    settings.theme === "light" || settings.theme === "dark"
      ? settings.theme
      : window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
  const material = getNativeMaterial(nativeMaterialEnabled);
  const windowTheme = getWindowTheme(settings.theme);
  const nativeAppearance = `${material}:${windowTheme}:${effectiveTheme}`;
  const nativeAppearanceChanged = lastNativeAppearance !== nativeAppearance;

  // 开启时先准备原生材质，避免 CSS 透明后短暂露出空白窗口。
  if (enablingMaterial && nativeAppearanceChanged) {
    await desktopApi.setWindowMaterial(material, windowTheme);
    lastNativeAppearance = nativeAppearance;
  }

  applyTheme(settings.theme || "auto");
  applyFontSize(settings.font_size || 14);
  applyFontFamily(settings.font_family || "");
  applyAcrylicEffect(nativeMaterialEnabled);
  if (!isThemeProviderActive()) {
    applyColors(settings);
  }

  // 关闭时先恢复 CSS 实色遮罩，再移除原生材质；主题切换也在 DOM 提交后同步。
  if (!enablingMaterial && nativeAppearanceChanged) {
    await desktopApi.setWindowMaterial(material, windowTheme);
    lastNativeAppearance = nativeAppearance;
  }
}

export function enqueueAppearanceApply(settings: AppSettings): Promise<void> {
  appearanceApplyQueue = appearanceApplyQueue.then(
    () => applyAppearance(settings),
    () => applyAppearance(settings),
  );
  return appearanceApplyQueue;
}
