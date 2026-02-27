import type { ThemeDefinition } from "@type/theme";

export const midnightTheme: ThemeDefinition = {
  id: "midnight",
  name: "Midnight",
  description: "午夜主题 - 深邃的蓝紫色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f8fafc",
    bgSecondary: "#eef2ff",
    bgTertiary: "#e0e7ff",
    primary: "#6366f1",
    secondary: "#8b5cf6",
    textPrimary: "#1e1b4b",
    textSecondary: "#4338ca",
    border: "#e0e7ff",
  },
  dark: {
    bg: "#0f0d1a",
    bgSecondary: "#1a1744",
    bgTertiary: "#252150",
    primary: "#818cf8",
    secondary: "#a78bfa",
    textPrimary: "#f5f3ff",
    textSecondary: "#c4b5fd",
    border: "rgba(139, 92, 246, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(248, 250, 252, 0.65)",
    bgSecondary: "rgba(238, 242, 255, 0.55)",
    bgTertiary: "rgba(224, 231, 255, 0.45)",
    primary: "#6366f1",
    secondary: "#8b5cf6",
    textPrimary: "#1e1b4b",
    textSecondary: "#4338ca",
    border: "rgba(224, 231, 255, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(15, 13, 26, 0.65)",
    bgSecondary: "rgba(26, 23, 68, 0.55)",
    bgTertiary: "rgba(37, 33, 80, 0.45)",
    primary: "#818cf8",
    secondary: "#a78bfa",
    textPrimary: "#f5f3ff",
    textSecondary: "#c4b5fd",
    border: "rgba(139, 92, 246, 0.1)",
  },
};

export default midnightTheme;
