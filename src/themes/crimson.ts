import type { ThemeDefinition } from "@type/theme";

export const crimsonTheme: ThemeDefinition = {
  id: "crimson",
  name: "Crimson",
  description: "绯红主题 - 热烈张扬的红色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fef2f2",
    bgSecondary: "#fee2e2",
    bgTertiary: "#fecaca",
    primary: "#dc2626",
    secondary: "#ef4444",
    textPrimary: "#7f1d1d",
    textSecondary: "#991b1b",
    border: "#fecaca",
  },
  dark: {
    bg: "#1c0606",
    bgSecondary: "#2f0d0d",
    bgTertiary: "#5b1616",
    primary: "#f87171",
    secondary: "#fca5a5",
    textPrimary: "#fef2f2",
    textSecondary: "#fecaca",
    border: "rgba(248, 113, 113, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(254, 242, 242, 0.65)",
    bgSecondary: "rgba(254, 226, 226, 0.55)",
    bgTertiary: "rgba(254, 202, 202, 0.45)",
    primary: "#dc2626",
    secondary: "#ef4444",
    textPrimary: "#7f1d1d",
    textSecondary: "#991b1b",
    border: "rgba(254, 202, 202, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(28, 6, 6, 0.65)",
    bgSecondary: "rgba(47, 13, 13, 0.55)",
    bgTertiary: "rgba(91, 22, 22, 0.45)",
    primary: "#f87171",
    secondary: "#fca5a5",
    textPrimary: "#fef2f2",
    textSecondary: "#fecaca",
    border: "rgba(248, 113, 113, 0.1)",
  },
};

export default crimsonTheme;
