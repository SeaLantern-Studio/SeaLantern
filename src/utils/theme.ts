/**
 * 主题相关工具函数
 * 提供主题、字体、颜色等通用处理功能
 */

import type { AppSettings } from "@api/settings";
import { isBrowserEnv } from "@api/tauri";
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

  // Surface and elevated backgrounds
  let surfaceColor: string;
  let surfaceHoverColor: string;
  let bgElevatedColor: string;
  let bgHoverColor: string;

  // Glass / Acrylic effect variables
  let glassBgColor: string;
  let glassStrongBgColor: string;
  let glassBorderColor: string;
  let acrylicBgColor: string;
  let acrylicBgStrongColor: string;
  let acrylicBorderColor: string;

  if (isAcrylic) {
    if (isDark) {
      // 暗色亚克力：深灰半透明层次
      surfaceColor = "rgba(42, 46, 62, 0.4)";
      surfaceHoverColor = "rgba(50, 55, 74, 0.48)";
      bgElevatedColor = "transparent";
      bgHoverColor = "rgba(255, 255, 255, 0.06)";
      // 卡片玻璃：较低透明度，保留通透感
      glassBgColor = "rgba(0, 0, 0, 0.3)";
      glassStrongBgColor = "rgba(0, 0, 0, 0.42)";
      glassBorderColor = "rgba(255, 255, 255, 0.06)";
      // 浮层（下拉菜单/弹出层）：较高透明度确保可读性
      acrylicBgColor = "rgba(30, 33, 48, 0.72)";
      acrylicBgStrongColor = "rgba(30, 33, 48, 0.85)";
      acrylicBorderColor = "rgba(255, 255, 255, 0.08)";
    } else {
      // 亮色亚克力：白色半透明层次
      surfaceColor = "rgba(255, 255, 255, 0.45)";
      surfaceHoverColor = "rgba(255, 255, 255, 0.52)";
      bgElevatedColor = "transparent";
      bgHoverColor = "rgba(255, 255, 255, 0.48)";
      // 卡片玻璃：较低透明度，叠加在 surface 上仍能透出壁纸
      glassBgColor = "rgba(255, 255, 255, 0.3)";
      glassStrongBgColor = "rgba(255, 255, 255, 0.42)";
      glassBorderColor = "rgba(15, 23, 42, 0.06)";
      // 浮层：较高透明度确保可读性
      acrylicBgColor = "rgba(255, 255, 255, 0.7)";
      acrylicBgStrongColor = "rgba(255, 255, 255, 0.82)";
      acrylicBorderColor = "rgba(15, 23, 42, 0.1)";
    }
  } else {
    // 非亚克力模式：实色背景
    surfaceColor = isDark ? colors.bgSecondary : "#ffffff";
    surfaceHoverColor = isDark ? colors.bgTertiary : colors.bg;
    bgElevatedColor = isDark ? colors.bgSecondary : "#ffffff";
    bgHoverColor = isDark ? colors.bgTertiary : colors.bgSecondary;
    glassBgColor = isDark ? "rgba(0, 0, 0, 0.72)" : "rgba(255, 255, 255, 0.72)";
    glassStrongBgColor = isDark ? "rgba(0, 0, 0, 0.82)" : "rgba(255, 255, 255, 0.82)";
    glassBorderColor = isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(15, 23, 42, 0.08)";
    acrylicBgColor = isDark ? "rgba(0, 0, 0, 0.85)" : "rgba(255, 255, 255, 0.92)";
    acrylicBgStrongColor = isDark ? "rgba(0, 0, 0, 0.92)" : "rgba(255, 255, 255, 0.96)";
    acrylicBorderColor = isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(15, 23, 42, 0.1)";
  }

  document.documentElement.style.setProperty("--sl-surface", surfaceColor);
  document.documentElement.style.setProperty("--sl-surface-hover", surfaceHoverColor);
  document.documentElement.style.setProperty("--sl-bg-elevated", bgElevatedColor);
  document.documentElement.style.setProperty("--sl-bg-hover", bgHoverColor);

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

  // 阴影：亚克力模式下更淡
  const shadowOpacity = isAcrylic ? (isDark ? 0.2 : 0.04) : isDark ? 0.4 : 0.06;
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
  // 立体感阴影：亚克力下更柔和
  const elevatedShadowOpacity = isAcrylic ? (isDark ? 0.25 : 0.06) : isDark ? 0.4 : 0.08;
  document.documentElement.style.setProperty(
    "--sl-shadow-elevated",
    `0 2px 8px rgba(0, 0, 0, ${elevatedShadowOpacity}), 0 4px 16px rgba(0, 0, 0, ${elevatedShadowOpacity * 0.75})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-card",
    `0 1px 4px rgba(0, 0, 0, ${shadowOpacity * 0.7}), 0 4px 12px rgba(0, 0, 0, ${shadowOpacity})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-button",
    `0 1px 3px rgba(0, 0, 0, ${shadowOpacity * 0.5}), 0 2px 6px rgba(0, 0, 0, ${shadowOpacity * 0.4})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-button-hover",
    `0 2px 6px rgba(0, 0, 0, ${shadowOpacity}), 0 4px 12px rgba(0, 0, 0, ${shadowOpacity * 0.7})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-input",
    `inset 0 1px 2px rgba(0, 0, 0, ${shadowOpacity * 0.5})`,
  );
  document.documentElement.style.setProperty(
    "--sl-shadow-input-focus",
    `0 0 0 3px ${isDark ? rgbaFromHex(colors.primary, 0.2) : rgbaFromHex(colors.primary, 0.15)}`,
  );

  // Glass / Acrylic 效果变量
  document.documentElement.style.setProperty("--sl-glass-bg", glassBgColor);
  document.documentElement.style.setProperty("--sl-glass-strong-bg", glassStrongBgColor);
  document.documentElement.style.setProperty("--sl-glass-border", glassBorderColor);
  document.documentElement.style.setProperty("--sl-acrylic-bg", acrylicBgColor);
  document.documentElement.style.setProperty("--sl-acrylic-bg-strong", acrylicBgStrongColor);
  document.documentElement.style.setProperty("--sl-acrylic-border", acrylicBorderColor);
}

/**
 * 应用开发者模式限制
 * @param enabled - 是否启用开发者模式
 */
export function applyDeveloperMode(enabled: boolean): void {
  // 在浏览器环境（Docker 模式）下，无法有效阻止开发者工具快捷键
  // 浏览器不允许网页完全禁用开发者工具，因此跳过限制逻辑
  // 这意味着 Docker 模式下开发者模式默认启用
  if (isBrowserEnv()) {
    return;
  }

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
 *
 * TODO: 请在后端重构完成后恢复（临时置空，开发调试用）
 * 拦截逻辑依赖后端 developer_mode 设置，但后端正在重构、
 * 暂时无法提供设置，为避免开发者模式下右键仍被拦截，
 * 暂时屏蔽此逻辑。恢复 `e.preventDefault()` 即可。
 */
function blockContextMenu(_e: Event): void {
  // TODO: 请在后端重构完成后恢复
  // e.preventDefault();
}

/**
 * 阻止开发者工具快捷键
 *
 * TODO: 请在后端重构完成后恢复（临时置空，开发调试用）
 * 原因同上，恢复 `e.preventDefault()` 即可。
 */
function blockDevTools(_e: KeyboardEvent): void {
  // TODO: 请在后端重构完成后恢复
  // if (e.key === "F12") {
  //   e.preventDefault();
  // }
}

/**
 * 应用极简模式到 DOM
 * 同步设置 data-animation 属性确保 CmzYa 组件库也响应动画关闭
 * @param enabled - 是否启用极简模式
 */
export function applyMinimalMode(enabled: boolean): void {
  document.documentElement.setAttribute("data-minimal", String(enabled));
  document.documentElement.setAttribute("data-animation", enabled ? "off" : "on");
}
