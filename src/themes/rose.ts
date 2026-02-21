import type { ThemeDefinition } from "../types";

export const roseTheme: ThemeDefinition = {
  id: "rose",
  name: "Rose",
  description: "玫瑰主题 - 浪漫的粉红色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fdf2f8",
    bgSecondary: "#fce7f3",
    bgTertiary: "#fbcfe8",
    primary: "#ec4899",
    secondary: "#db2777",
    textPrimary: "#831843",
    textSecondary: "#9f1239",
    border: "#fce7f3",
  },
  dark: {
    bg: "#831843",
    bgSecondary: "#9f1239",
    bgTertiary: "#be123c",
    primary: "#f472b6",
    secondary: "#f9a8d4",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(253, 242, 248, 0.7)",
    bgSecondary: "rgba(252, 231, 243, 0.6)",
    bgTertiary: "rgba(251, 207, 232, 0.5)",
    primary: "#ec4899",
    secondary: "#db2777",
    textPrimary: "#831843",
    textSecondary: "#9f1239",
    border: "#fce7f3",
  },
  darkAcrylic: {
    bg: "rgba(131, 24, 67, 0.7)",
    bgSecondary: "rgba(159, 18, 57, 0.6)",
    bgTertiary: "rgba(190, 18, 60, 0.5)",
    primary: "#f472b6",
    secondary: "#f9a8d4",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default roseTheme;
