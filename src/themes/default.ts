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
    bg: "#0f1117",
    bgSecondary: "#1a1d28",
    bgTertiary: "#242836",
    primary: "#60a5fa",
    secondary: "#22d3ee",
    textPrimary: "#e2e8f0",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(248, 250, 252, 0.7)",
    bgSecondary: "rgba(241, 245, 249, 0.6)",
    bgTertiary: "rgba(226, 232, 240, 0.5)",
    primary: "#0ea5e9",
    secondary: "#06b6d4",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  darkAcrylic: {
    bg: "rgba(15, 17, 23, 0.7)",
    bgSecondary: "rgba(26, 29, 40, 0.6)",
    bgTertiary: "rgba(36, 40, 54, 0.5)",
    primary: "#60a5fa",
    secondary: "#22d3ee",
    textPrimary: "#e2e8f0",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default defaultTheme;
