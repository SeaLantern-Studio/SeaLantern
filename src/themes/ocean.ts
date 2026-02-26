import type { ThemeDefinition } from "@type/theme";

export const oceanTheme: ThemeDefinition = {
  id: "ocean",
  name: "Ocean",
  description: "海洋主题 - 清爽的青蓝色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f0fdfa",
    bgSecondary: "#ccfbf1",
    bgTertiary: "#99f6e4",
    primary: "#0d9488",
    secondary: "#0891b2",
    textPrimary: "#134e4a",
    textSecondary: "#0f766e",
    border: "#99f6e4",
  },
  dark: {
    bg: "#0a1929",
    bgSecondary: "#0d2847",
    bgTertiary: "#134e6f",
    primary: "#2dd4bf",
    secondary: "#22d3ee",
    textPrimary: "#f0fdfa",
    textSecondary: "#99f6e4",
    border: "rgba(45, 212, 191, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(240, 253, 250, 0.65)",
    bgSecondary: "rgba(204, 251, 241, 0.55)",
    bgTertiary: "rgba(153, 246, 228, 0.45)",
    primary: "#0d9488",
    secondary: "#0891b2",
    textPrimary: "#134e4a",
    textSecondary: "#0f766e",
    border: "rgba(153, 246, 228, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(10, 25, 41, 0.65)",
    bgSecondary: "rgba(13, 40, 71, 0.55)",
    bgTertiary: "rgba(19, 78, 111, 0.45)",
    primary: "#2dd4bf",
    secondary: "#22d3ee",
    textPrimary: "#f0fdfa",
    textSecondary: "#99f6e4",
    border: "rgba(45, 212, 191, 0.1)",
  },
};

export default oceanTheme;
