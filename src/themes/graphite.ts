import type { ThemeDefinition } from "@type/theme";

export const graphiteTheme: ThemeDefinition = {
  id: "graphite",
  name: "Graphite",
  description: "石墨主题 - 冷静克制的纯灰调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#fafafa",
    bgSecondary: "#f4f4f5",
    bgTertiary: "#e4e4e7",
    primary: "#52525b",
    secondary: "#71717a",
    textPrimary: "#18181b",
    textSecondary: "#3f3f46",
    border: "#e4e4e7",
  },
  dark: {
    bg: "#0e0e10",
    bgSecondary: "#18181b",
    bgTertiary: "#27272a",
    primary: "#a1a1aa",
    secondary: "#d4d4d8",
    textPrimary: "#fafafa",
    textSecondary: "#a1a1aa",
    border: "rgba(161, 161, 170, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(250, 250, 250, 0.65)",
    bgSecondary: "rgba(244, 244, 245, 0.55)",
    bgTertiary: "rgba(228, 228, 231, 0.45)",
    primary: "#52525b",
    secondary: "#71717a",
    textPrimary: "#18181b",
    textSecondary: "#3f3f46",
    border: "rgba(228, 228, 231, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(14, 14, 16, 0.65)",
    bgSecondary: "rgba(24, 24, 27, 0.55)",
    bgTertiary: "rgba(39, 39, 42, 0.45)",
    primary: "#a1a1aa",
    secondary: "#d4d4d8",
    textPrimary: "#fafafa",
    textSecondary: "#a1a1aa",
    border: "rgba(161, 161, 170, 0.1)",
  },
};

export default graphiteTheme;
