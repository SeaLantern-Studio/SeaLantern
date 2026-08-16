import type { ThemeDefinition } from "@type/theme";

export const limeTheme: ThemeDefinition = {
  id: "lime",
  name: "Lime",
  description: "柠檬主题 - 明快活泼的黄绿色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f7fee7",
    bgSecondary: "#ecfccb",
    bgTertiary: "#d9f99d",
    primary: "#65a30d",
    secondary: "#84cc16",
    textPrimary: "#365314",
    textSecondary: "#4d7c0f",
    border: "#d9f99d",
  },
  dark: {
    bg: "#141a04",
    bgSecondary: "#263308",
    bgTertiary: "#43520e",
    primary: "#a3e635",
    secondary: "#bef264",
    textPrimary: "#f7fee7",
    textSecondary: "#d9f99d",
    border: "rgba(163, 230, 53, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(247, 254, 231, 0.65)",
    bgSecondary: "rgba(236, 252, 203, 0.55)",
    bgTertiary: "rgba(217, 249, 157, 0.45)",
    primary: "#65a30d",
    secondary: "#84cc16",
    textPrimary: "#365314",
    textSecondary: "#4d7c0f",
    border: "rgba(217, 249, 157, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(20, 26, 4, 0.65)",
    bgSecondary: "rgba(38, 51, 8, 0.55)",
    bgTertiary: "rgba(67, 82, 14, 0.45)",
    primary: "#a3e635",
    secondary: "#bef264",
    textPrimary: "#f7fee7",
    textSecondary: "#d9f99d",
    border: "rgba(163, 230, 53, 0.1)",
  },
};

export default limeTheme;
