import type { ThemeDefinition } from "@type/theme";

export const emeraldTheme: ThemeDefinition = {
  id: "emerald",
  name: "Emerald",
  description: "翡翠主题 - 清新自然的绿色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#ecfdf5",
    bgSecondary: "#d1fae5",
    bgTertiary: "#a7f3d0",
    primary: "#059669",
    secondary: "#10b981",
    textPrimary: "#064e3b",
    textSecondary: "#047857",
    border: "#a7f3d0",
  },
  dark: {
    bg: "#04150f",
    bgSecondary: "#0c2a1f",
    bgTertiary: "#14532d",
    primary: "#4ade80",
    secondary: "#34d399",
    textPrimary: "#ecfdf5",
    textSecondary: "#a7f3d0",
    border: "rgba(74, 222, 128, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(236, 253, 245, 0.65)",
    bgSecondary: "rgba(209, 250, 229, 0.55)",
    bgTertiary: "rgba(167, 243, 208, 0.45)",
    primary: "#059669",
    secondary: "#10b981",
    textPrimary: "#064e3b",
    textSecondary: "#047857",
    border: "rgba(167, 243, 208, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(4, 21, 15, 0.65)",
    bgSecondary: "rgba(12, 42, 31, 0.55)",
    bgTertiary: "rgba(20, 83, 45, 0.45)",
    primary: "#4ade80",
    secondary: "#34d399",
    textPrimary: "#ecfdf5",
    textSecondary: "#a7f3d0",
    border: "rgba(74, 222, 128, 0.1)",
  },
};

export default emeraldTheme;
