import type { ThemeDefinition } from "@type/theme";

export const cyanTheme: ThemeDefinition = {
  id: "cyan",
  name: "Cyan",
  description: "青色主题 - 澄澈通透的蓝青色",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#ecfeff",
    bgSecondary: "#cffafe",
    bgTertiary: "#a5f3fc",
    primary: "#06b6d4",
    secondary: "#0891b2",
    textPrimary: "#164e63",
    textSecondary: "#0e7490",
    border: "#a5f3fc",
  },
  dark: {
    bg: "#082f49",
    bgSecondary: "#0c4a6e",
    bgTertiary: "#155e75",
    primary: "#22d3ee",
    secondary: "#67e8f9",
    textPrimary: "#ecfeff",
    textSecondary: "#a5f3fc",
    border: "rgba(34, 211, 238, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(236, 254, 255, 0.65)",
    bgSecondary: "rgba(207, 250, 254, 0.55)",
    bgTertiary: "rgba(165, 243, 252, 0.45)",
    primary: "#06b6d4",
    secondary: "#0891b2",
    textPrimary: "#164e63",
    textSecondary: "#0e7490",
    border: "rgba(165, 243, 252, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(8, 47, 73, 0.65)",
    bgSecondary: "rgba(12, 74, 110, 0.55)",
    bgTertiary: "rgba(21, 94, 117, 0.45)",
    primary: "#22d3ee",
    secondary: "#67e8f9",
    textPrimary: "#ecfeff",
    textSecondary: "#a5f3fc",
    border: "rgba(34, 211, 238, 0.1)",
  },
};

export default cyanTheme;
