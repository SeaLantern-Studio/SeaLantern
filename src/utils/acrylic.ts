import { DEFAULT_ACRYLIC_BLUR_LEVEL, type AcrylicBlurLevel } from "@api/settings";

export const ACRYLIC_BLUR_VALUES: Record<AcrylicBlurLevel, string> = {
  off: "0px",
  low: "8px",
  medium: "16px",
  high: "28px",
};

export function normalizeAcrylicBlurLevel(level?: string): AcrylicBlurLevel {
  return level === "off" || level === "low" || level === "medium" || level === "high"
    ? level
    : DEFAULT_ACRYLIC_BLUR_LEVEL;
}

export function applyAcrylicEffect(enabled: boolean, level?: string): void {
  const root = document.documentElement;

  root.setAttribute("data-acrylic", enabled ? "on" : "off");

  // 关闭时清理模糊属性和变量，避免残留值误导其他依赖方
  if (!enabled) {
    root.removeAttribute("data-acrylic-blur");
    root.style.removeProperty("--sl-acrylic-blur");
    return;
  }

  const normalizedLevel = normalizeAcrylicBlurLevel(level);
  root.setAttribute("data-acrylic-blur", normalizedLevel);
  root.style.setProperty("--sl-acrylic-blur", ACRYLIC_BLUR_VALUES[normalizedLevel]);
}
