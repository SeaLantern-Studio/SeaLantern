import type { ThemeDefinition } from "../types";

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
    primary: "#f97316",
    secondary: "#ea580c",
    textPrimary: "#7c2d12",
    textSecondary: "#9a3412",
    border: "#fef3c7",
  },
  dark: {
    bg: "#7c2d12",
    bgSecondary: "#9a3412",
    bgTertiary: "#b45309",
    primary: "#fb923c",
    secondary: "#fdba74",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(255, 251, 235, 0.7)",
    bgSecondary: "rgba(254, 243, 199, 0.6)",
    bgTertiary: "rgba(253, 230, 138, 0.5)",
    primary: "#f97316",
    secondary: "#ea580c",
    textPrimary: "#7c2d12",
    textSecondary: "#9a3412",
    border: "#fef3c7",
  },
  darkAcrylic: {
    bg: "rgba(124, 45, 18, 0.7)",
    bgSecondary: "rgba(154, 52, 18, 0.6)",
    bgTertiary: "rgba(180, 83, 9, 0.5)",
    primary: "#fb923c",
    secondary: "#fdba74",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default sunsetTheme;
