import type { ThemeDefinition } from "@type/theme";

export const coffeeTheme: ThemeDefinition = {
  id: "coffee",
  name: "Coffee",
  description: "咖啡主题 - 醇厚温暖的棕色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#faf6f0",
    bgSecondary: "#f2ece3",
    bgTertiary: "#e5d9c8",
    primary: "#92400e",
    secondary: "#a16207",
    textPrimary: "#3f2412",
    textSecondary: "#6f4a24",
    border: "#e5d9c8",
  },
  dark: {
    bg: "#1a1209",
    bgSecondary: "#2d2012",
    bgTertiary: "#4a3418",
    primary: "#d97706",
    secondary: "#f59e0b",
    textPrimary: "#faf6f0",
    textSecondary: "#e5d9c8",
    border: "rgba(217, 119, 6, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(250, 246, 240, 0.65)",
    bgSecondary: "rgba(242, 236, 227, 0.55)",
    bgTertiary: "rgba(229, 217, 200, 0.45)",
    primary: "#92400e",
    secondary: "#a16207",
    textPrimary: "#3f2412",
    textSecondary: "#6f4a24",
    border: "rgba(229, 217, 200, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(26, 18, 9, 0.65)",
    bgSecondary: "rgba(45, 32, 18, 0.55)",
    bgTertiary: "rgba(74, 52, 24, 0.45)",
    primary: "#d97706",
    secondary: "#f59e0b",
    textPrimary: "#faf6f0",
    textSecondary: "#e5d9c8",
    border: "rgba(217, 119, 6, 0.1)",
  },
};

export default coffeeTheme;
