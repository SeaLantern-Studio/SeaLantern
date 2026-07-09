import type { ThemeDefinition } from "@type/theme";

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
    primary: "#db2777",
    secondary: "#ec4899",
    textPrimary: "#831843",
    textSecondary: "#9f1239",
    border: "#fbcfe8",
  },
  dark: {
    bg: "#1a0a12",
    bgSecondary: "#2d1220",
    bgTertiary: "#4a1942",
    primary: "#f472b6",
    secondary: "#fb7185",
    textPrimary: "#fdf2f8",
    textSecondary: "#fbcfe8",
    border: "rgba(244, 114, 182, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(253, 242, 248, 0.65)",
    bgSecondary: "rgba(252, 231, 243, 0.55)",
    bgTertiary: "rgba(251, 207, 232, 0.45)",
    primary: "#db2777",
    secondary: "#ec4899",
    textPrimary: "#831843",
    textSecondary: "#9f1239",
    border: "rgba(251, 207, 232, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(26, 10, 18, 0.65)",
    bgSecondary: "rgba(45, 18, 32, 0.55)",
    bgTertiary: "rgba(74, 25, 66, 0.45)",
    primary: "#f472b6",
    secondary: "#fb7185",
    textPrimary: "#fdf2f8",
    textSecondary: "#fbcfe8",
    border: "rgba(244, 114, 182, 0.1)",
  },
};

export default roseTheme;
