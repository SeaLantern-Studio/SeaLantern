import type { ThemeDefinition } from "../types";

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
    primary: "#06b6d4",
    secondary: "#0891b2",
    textPrimary: "#0e7490",
    textSecondary: "#155e75",
    border: "#ccfbf1",
  },
  dark: {
    bg: "#0e7490",
    bgSecondary: "#0c4a6e",
    bgTertiary: "#0891b2",
    primary: "#22d3ee",
    secondary: "#67e8f9",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(240, 253, 250, 0.7)",
    bgSecondary: "rgba(204, 251, 241, 0.6)",
    bgTertiary: "rgba(153, 246, 228, 0.5)",
    primary: "#06b6d4",
    secondary: "#0891b2",
    textPrimary: "#0e7490",
    textSecondary: "#155e75",
    border: "#ccfbf1",
  },
  darkAcrylic: {
    bg: "rgba(14, 116, 144, 0.7)",
    bgSecondary: "rgba(12, 74, 110, 0.6)",
    bgTertiary: "rgba(8, 145, 178, 0.5)",
    primary: "#22d3ee",
    secondary: "#67e8f9",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default oceanTheme;
