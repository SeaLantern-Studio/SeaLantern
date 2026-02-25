/**
 * 主题相关工具函数
 * 提供主题、字体、颜色等通用处理功能
 */

import type { AppSettings } from "@api/settings";
import { getThemeColors, mapLegacyPlanName } from "@themes";

let _themeProviderOverrides: string[] = [];

export function setThemeProviderOverrides(overrides: string[]): void {
  _themeProviderOverrides = Array.isArray(overrides) ? overrides : [];
}

export function isThemeProviderActive(): boolean {
  return _themeProviderOverrides.length > 0;
}

/**
 * 获取实际生效的主题（light 或 dark）
 * @param theme - 主题设置值，可以是 "light"、"dark" 或 "auto"
 * @returns 实际生效的主题
 */
export function getEffectiveTheme(theme: string): "light" | "dark" {
  if (theme === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme as "light" | "dark";
}

/**
 * 应用主题到 DOM
 * @param theme - 主题设置值
 * @returns 实际生效的主题
 */
export function applyTheme(theme: string): "light" | "dark" {
  const effectiveTheme = getEffectiveTheme(theme);
  document.documentElement.setAttribute("data-theme", effectiveTheme);
  return effectiveTheme;
}

/**
 * 应用字体设置到 DOM
 * @param fontFamily - 字体名称，为空则移除自定义字体
 */
export function applyFontFamily(fontFamily: string): void {
  if (fontFamily) {
    document.documentElement.style.setProperty("--sl-font-sans", fontFamily);
    document.documentElement.style.setProperty("--sl-font-display", fontFamily);
  } else {
    document.documentElement.style.removeProperty("--sl-font-sans");
    document.documentElement.style.removeProperty("--sl-font-display");
  }
}

/**
 * 应用字体大小到 DOM
 * @param fontSize - 字体大小（像素值）
 */
export function applyFontSize(fontSize: number): void {
  document.documentElement.style.fontSize = fontSize + "px";
}

/**
 * 调整十六进制颜色的亮度
 * @param hex - 十六进制颜色值
 * @param percent - 调整百分比，正数变亮，负数变暗
 * @returns 调整后的十六进制颜色值
 */
export function adjustBrightness(hex: string, percent: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const amt = Math.round(2.55 * percent);
  const R = (num >> 16) + amt;
  const G = ((num >> 8) & 0x00ff) + amt;
  const B = (num & 0x0000ff) + amt;
  return (
    "#" +
    (
      0x1000000 +
      (R < 255 ? (R < 1 ? 0 : R) : 255) * 0x10000 +
      (G < 255 ? (G < 1 ? 0 : G) : 255) * 0x100 +
      (B < 255 ? (B < 1 ? 0 : B) : 255)
    )
      .toString(16)
      .slice(1)
  );
}

/**
 * 将十六进制颜色转换为 RGBA 格式
 * @param hex - 十六进制颜色值
 * @param alpha - 透明度（0-1）
 * @returns RGBA 格式的颜色字符串
 */
export function rgbaFromHex(hex: string, alpha: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const R = (num >> 16) & 0xff;
  const G = (num >> 8) & 0xff;
  const B = num & 0xff;
  return `rgba(${R}, ${G}, ${B}, ${alpha})`;
}

/**
 * 获取指定主题方案下的颜色值
 * @param settings - 应用设置
 * @param colorType - 颜色类型
 * @param theme - 主题方案名称
 * @returns 颜色值字符串
 */
export function getColorValue(settings: AppSettings, colorType: string, theme: string): string {
  if (!settings) return "";

  const plan = mapLegacyPlanName(theme);
  const themeColors = getThemeColors(settings.color, plan);
  if (themeColors) {
    return themeColors[colorType as keyof typeof themeColors] || "";
  }
  return "";
}

/**
 * 应用颜色设置到 DOM
 * @param settings - 应用设置
 */
export function applyColors(settings: AppSettings): void {
  if (!settings) return;

  if (_themeProviderOverrides.length > 0) {
    return;
  }

  const effectiveTheme = getEffectiveTheme(settings.theme);
  const isDark = effectiveTheme === "dark";
  const isAcrylic = settings.acrylic_enabled;

  const actualPlan = isDark
    ? isAcrylic
      ? "dark_acrylic"
      : "dark"
    : isAcrylic
      ? "light_acrylic"
      : "light";

  const colors = {
    bg: getColorValue(settings, "bg", actualPlan),
    bgSecondary: getColorValue(settings, "bgSecondary", actualPlan),
    bgTertiary: getColorValue(settings, "bgTertiary", actualPlan),
    primary: getColorValue(settings, "primary", actualPlan),
    secondary: getColorValue(settings, "secondary", actualPlan),
    textPrimary: getColorValue(settings, "textPrimary", actualPlan),
    textSecondary: getColorValue(settings, "textSecondary", actualPlan),
    border: getColorValue(settings, "border", actualPlan),
  };

  document.documentElement.style.setProperty("--sl-bg", colors.bg);
  document.documentElement.style.setProperty("--sl-bg-secondary", colors.bgSecondary);
  document.documentElement.style.setProperty("--sl-bg-tertiary", colors.bgTertiary);
  document.documentElement.style.setProperty("--sl-primary", colors.primary);
  document.documentElement.style.setProperty("--sl-accent", colors.secondary);
  document.documentElement.style.setProperty("--sl-text-primary", colors.textPrimary);
  document.documentElement.style.setProperty("--sl-text-secondary", colors.textSecondary);
  document.documentElement.style.setProperty("--sl-border", colors.border);
  document.documentElement.style.setProperty("--sl-border-light", colors.border);

  let surfaceColor: string;
  let surfaceHoverColor: string;
  if (isAcrylic) {
    if (isDark) {
      surfaceColor = "rgba(30, 33, 48, 0.6)";
      surfaceHoverColor = "rgba(40, 44, 62, 0.7)";
    } else {
      surfaceColor = "rgba(255, 255, 255, 0.6)";
      surfaceHoverColor = "rgba(248, 250, 252, 0.7)";
    }
  } else {
    surfaceColor = isDark ? colors.bgSecondary : "#ffffff";
    surfaceHoverColor = isDark ? colors.bgTertiary : colors.bg;
  }
  document.documentElement.style.setProperty("--sl-surface", surfaceColor);
  document.documentElement.style.setProperty("--sl-surface-hover", surfaceHoverColor);

  const primaryLight = isDark
    ? adjustBrightness(colors.primary, 30)
    : adjustBrightness(colors.primary, 20);
  const primaryDark = isDark
    ? adjustBrightness(colors.primary, -20)
    : adjustBrightness(colors.primary, -30);
  const primaryBg = isDark ? rgbaFromHex(colors.primary, 0.12) : rgbaFromHex(colors.primary, 0.08);
  document.documentElement.style.setProperty("--sl-primary-light", primaryLight);
  document.documentElement.style.setProperty("--sl-primary-dark", primaryDark);
  document.documentElement.style.setProperty("--sl-primary-bg", primaryBg);

  const accentLight = adjustBrightness(colors.secondary, 20);
  document.documentElement.style.setProperty("--sl-accent-light", accentLight);

  const textTertiary = isDark
    ? adjustBrightness(colors.textSecondary, -20)
    : adjustBrightness(colors.textSecondary, 20);
  const textInverse = "#ffffff";
  document.documentElement.style.setProperty("--sl-text-tertiary", textTertiary);
  document.documentElement.style.setProperty("--sl-text-inverse", textInverse);

  const shadowOpacity = isDark ? 0.4 : 0.06;
  document.documentElement.style.setProperty(
    "--sl-shadow-sm",
    `0 1px 2px rgba(0, 0, 0, ${shadowOpacity * 0.6})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-md",
    `0 4px 12px rgba(0, 0, 0, ${shadowOpacity})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-lg",
    `0 8px 24px rgba(0, 0, 0, ${shadowOpacity * 1.3})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-xl",
    `0 16px 48px rgba(0, 0, 0, ${shadowOpacity * 1.6})`,
  );

  // Glass 效果变量
  const glassBg = isDark ? "rgba(15, 17, 23, 0.72)" : "rgba(255, 255, 255, 0.72)";
  const glassStrongBg = isDark ? "rgba(15, 17, 23, 0.88)" : "rgba(255, 255, 255, 0.88)";
  const glassBorder = isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(255, 255, 255, 0.5)";
  document.documentElement.style.setProperty("--sl-glass-bg", glassBg);
  document.documentElement.style.setProperty("--sl-glass-strong-bg", glassStrongBg);
  document.documentElement.style.setProperty("--sl-glass-border", glassBorder);
}

/**
 * 应用开发者模式限制
 * @param enabled - 是否启用开发者模式
 */
export function applyDeveloperMode(enabled: boolean): void {
  if (enabled) {
    document.removeEventListener("contextmenu", blockContextMenu);
    document.removeEventListener("keydown", blockDevTools);
  } else {
    document.addEventListener("contextmenu", blockContextMenu);
    document.addEventListener("keydown", blockDevTools);
  }
}

/**
 * 阻止右键菜单
 */
function blockContextMenu(e: Event): void {
  e.preventDefault();
}

/**
 * 阻止开发者工具快捷键
 */
function blockDevTools(e: KeyboardEvent): void {
  if (e.key === "F12") {
    e.preventDefault();
  }
}

/**
 * 应用极简模式到 DOM
 * @param enabled - 是否启用极简模式
 */
export function applyMinimalMode(enabled: boolean): void {
  document.documentElement.setAttribute("data-minimal", String(enabled));
}
