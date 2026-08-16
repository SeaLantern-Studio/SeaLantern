import type { ThemeDefinition } from "@type/theme";

export const fuchsiaTheme: ThemeDefinition = {
  id: "fuchsia",
  name: "Fuchsia",
  description: "品红主题 - 前卫亮眼的洋红色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fdf4ff",
    bgSecondary: "#fae8ff",
    bgTertiary: "#f5d0fe",
    primary: "#c026d3",
    secondary: "#d946ef",
    textPrimary: "#701a75",
    textSecondary: "#86198f",
    border: "#f5d0fe",
  },
  dark: {
    bg: "#1c071f",
    bgSecondary: "#31103a",
    bgTertiary: "#5b1d63",
    primary: "#e879f9",
    secondary: "#f0abfc",
    textPrimary: "#fdf4ff",
    textSecondary: "#f5d0fe",
    border: "rgba(232, 121, 249, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(253, 244, 255, 0.65)",
    bgSecondary: "rgba(250, 232, 255, 0.55)",
    bgTertiary: "rgba(245, 208, 254, 0.45)",
    primary: "#c026d3",
    secondary: "#d946ef",
    textPrimary: "#701a75",
    textSecondary: "#86198f",
    border: "rgba(245, 208, 254, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(28, 7, 31, 0.65)",
    bgSecondary: "rgba(49, 16, 58, 0.55)",
    bgTertiary: "rgba(91, 29, 99, 0.45)",
    primary: "#e879f9",
    secondary: "#f0abfc",
    textPrimary: "#fdf4ff",
    textSecondary: "#f5d0fe",
    border: "rgba(232, 121, 249, 0.1)",
  },
};

export default fuchsiaTheme;
