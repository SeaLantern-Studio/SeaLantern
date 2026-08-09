import type { AcrylicBlurLevel } from "@api/settings";

export const ACRYLIC_BLUR_VALUES: Record<AcrylicBlurLevel, string> = {
  off: "0px",
  low: "8px",
  medium: "16px",
  high: "28px",
};

export function normalizeAcrylicBlurLevel(level?: string): AcrylicBlurLevel {
  return level === "off" || level === "low" || level === "medium" || level === "high"
    ? level
    : "medium";
}

export function applyAcrylicEffect(enabled: boolean, level?: string): void {
  const normalizedLevel = normalizeAcrylicBlurLevel(level);
  const root = document.documentElement;

  root.setAttribute("data-acrylic", enabled ? "on" : "off");
  root.setAttribute("data-acrylic-blur", normalizedLevel);
  root.style.setProperty(
    "--sl-acrylic-blur",
    enabled ? ACRYLIC_BLUR_VALUES[normalizedLevel] : "0px",
  );
}
