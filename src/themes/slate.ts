import type { ThemeDefinition } from "@type/theme";

export const slateTheme: ThemeDefinition = {
  id: "slate",
  name: "Slate",
  description: "石墨主题 - 沉稳克制的蓝灰色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f1f5f9",
    bgSecondary: "#e2e8f0",
    bgTertiary: "#cbd5e1",
    primary: "#475569",
    secondary: "#64748b",
    textPrimary: "#1e293b",
    textSecondary: "#334155",
    border: "#cbd5e1",
  },
  dark: {
    bg: "#0b1120",
    bgSecondary: "#131c2e",
    bgTertiary: "#1e2a3d",
    primary: "#94a3b8",
    secondary: "#cbd5e1",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(148, 163, 184, 0.15)",
  },
  lightAcrylic: {
    bg: "rgba(241, 245, 249, 0.65)",
    bgSecondary: "rgba(226, 232, 240, 0.55)",
    bgTertiary: "rgba(203, 213, 225, 0.45)",
    primary: "#475569",
    secondary: "#64748b",
    textPrimary: "#1e293b",
    textSecondary: "#334155",
    border: "rgba(203, 213, 225, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(11, 17, 32, 0.65)",
    bgSecondary: "rgba(19, 28, 46, 0.55)",
    bgTertiary: "rgba(30, 42, 61, 0.45)",
    primary: "#94a3b8",
    secondary: "#cbd5e1",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(148, 163, 184, 0.1)",
  },
};

export default slateTheme;
