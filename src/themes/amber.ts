import type { ThemeDefinition } from "@type/theme";

export const amberTheme: ThemeDefinition = {
  id: "amber",
  name: "Amber",
  description: "琥珀主题 - 温暖明亮的金色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fffbeb",
    bgSecondary: "#fef3c7",
    bgTertiary: "#fde68a",
    primary: "#d97706",
    secondary: "#f59e0b",
    textPrimary: "#78350f",
    textSecondary: "#92400e",
    border: "#fde68a",
  },
  dark: {
    bg: "#1c1204",
    bgSecondary: "#33220a",
    bgTertiary: "#573a10",
    primary: "#fbbf24",
    secondary: "#fcd34d",
    textPrimary: "#fffbeb",
    textSecondary: "#fde68a",
    border: "rgba(251, 191, 36, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(255, 251, 235, 0.65)",
    bgSecondary: "rgba(254, 243, 199, 0.55)",
    bgTertiary: "rgba(253, 230, 138, 0.45)",
    primary: "#d97706",
    secondary: "#f59e0b",
    textPrimary: "#78350f",
    textSecondary: "#92400e",
    border: "rgba(253, 230, 138, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(28, 18, 4, 0.65)",
    bgSecondary: "rgba(51, 34, 10, 0.55)",
    bgTertiary: "rgba(87, 58, 16, 0.45)",
    primary: "#fbbf24",
    secondary: "#fcd34d",
    textPrimary: "#fffbeb",
    textSecondary: "#fde68a",
    border: "rgba(251, 191, 36, 0.1)",
  },
};

export default amberTheme;
