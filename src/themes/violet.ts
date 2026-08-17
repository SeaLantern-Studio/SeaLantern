import type { ThemeDefinition } from "@type/theme";

export const violetTheme: ThemeDefinition = {
  id: "violet",
  name: "Violet",
  description: "紫罗兰主题 - 神秘优雅的纯紫色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#faf5ff",
    bgSecondary: "#f3e8ff",
    bgTertiary: "#e9d5ff",
    primary: "#9333ea",
    secondary: "#a855f7",
    textPrimary: "#581c87",
    textSecondary: "#7e22ce",
    border: "#e9d5ff",
  },
  dark: {
    bg: "#170a26",
    bgSecondary: "#241040",
    bgTertiary: "#3b1d6e",
    primary: "#c084fc",
    secondary: "#d8b4fe",
    textPrimary: "#faf5ff",
    textSecondary: "#e9d5ff",
    border: "rgba(192, 132, 252, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(250, 245, 255, 0.65)",
    bgSecondary: "rgba(243, 232, 255, 0.55)",
    bgTertiary: "rgba(233, 213, 255, 0.45)",
    primary: "#9333ea",
    secondary: "#a855f7",
    textPrimary: "#581c87",
    textSecondary: "#7e22ce",
    border: "rgba(233, 213, 255, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(23, 10, 38, 0.65)",
    bgSecondary: "rgba(36, 16, 64, 0.55)",
    bgTertiary: "rgba(59, 29, 110, 0.45)",
    primary: "#c084fc",
    secondary: "#d8b4fe",
    textPrimary: "#faf5ff",
    textSecondary: "#e9d5ff",
    border: "rgba(192, 132, 252, 0.1)",
  },
};

export default violetTheme;
