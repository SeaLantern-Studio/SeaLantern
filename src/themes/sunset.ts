import type { ThemeDefinition } from "@type/theme";

export const sunsetTheme: ThemeDefinition = {
  id: "sunset",
  name: "Sunset",
  description: "日落主题 - 温暖的橙黄色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fffbeb",
    bgSecondary: "#fef3c7",
    bgTertiary: "#fde68a",
    primary: "#ea580c",
    secondary: "#f97316",
    textPrimary: "#7c2d12",
    textSecondary: "#9a3412",
    border: "#fde68a",
  },
  dark: {
    bg: "#1a0f05",
    bgSecondary: "#2d1a0a",
    bgTertiary: "#4a2c12",
    primary: "#fb923c",
    secondary: "#fbbf24",
    textPrimary: "#fffbeb",
    textSecondary: "#fef3c7",
    border: "rgba(251, 146, 60, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(255, 251, 235, 0.65)",
    bgSecondary: "rgba(254, 243, 199, 0.55)",
    bgTertiary: "rgba(253, 230, 138, 0.45)",
    primary: "#ea580c",
    secondary: "#f97316",
    textPrimary: "#7c2d12",
    textSecondary: "#9a3412",
    border: "rgba(253, 230, 138, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(26, 15, 5, 0.65)",
    bgSecondary: "rgba(45, 26, 10, 0.55)",
    bgTertiary: "rgba(74, 44, 18, 0.45)",
    primary: "#fb923c",
    secondary: "#fbbf24",
    textPrimary: "#fffbeb",
    textSecondary: "#fef3c7",
    border: "rgba(251, 146, 60, 0.1)",
  },
};

export default sunsetTheme;
