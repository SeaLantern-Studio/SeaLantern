import type { ThemeDefinition } from "@type/theme";

export const defaultTheme: ThemeDefinition = {
  id: "default",
  name: "Default",
  description: "SeaLantern 默认主题 - 清新简洁的蓝绿色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f8fafc",
    bgSecondary: "#f1f5f9",
    bgTertiary: "#e2e8f0",
    primary: "#0ea5e9",
    secondary: "#06b6d4",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  dark: {
    bg: "#0c1222",
    bgSecondary: "#151d2e",
    bgTertiary: "#1e293b",
    primary: "#38bdf8",
    secondary: "#22d3ee",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.08)",
  },
  lightAcrylic: {
    bg: "rgba(248, 250, 252, 0.65)",
    bgSecondary: "rgba(241, 245, 249, 0.55)",
    bgTertiary: "rgba(226, 232, 240, 0.45)",
    primary: "#0ea5e9",
    secondary: "#06b6d4",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "rgba(226, 232, 240, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(12, 18, 34, 0.65)",
    bgSecondary: "rgba(21, 29, 46, 0.55)",
    bgTertiary: "rgba(30, 41, 59, 0.45)",
    primary: "#38bdf8",
    secondary: "#22d3ee",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.06)",
  },
};

export default defaultTheme;
