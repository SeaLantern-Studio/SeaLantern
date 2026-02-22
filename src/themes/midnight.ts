import type { ThemeDefinition } from "@type/theme";

export const midnightTheme: ThemeDefinition = {
  id: "midnight",
  name: "Midnight",
  description: "午夜主题 - 深邃的蓝紫色调",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f0f4f8",
    bgSecondary: "#e2e8f0",
    bgTertiary: "#cbd5e1",
    primary: "#3b82f6",
    secondary: "#6366f1",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  dark: {
    bg: "#0f172a",
    bgSecondary: "#1e293b",
    bgTertiary: "#334155",
    primary: "#60a5fa",
    secondary: "#818cf8",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(240, 244, 248, 0.7)",
    bgSecondary: "rgba(226, 232, 240, 0.6)",
    bgTertiary: "rgba(203, 213, 225, 0.5)",
    primary: "#3b82f6",
    secondary: "#6366f1",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  darkAcrylic: {
    bg: "rgba(15, 23, 42, 0.7)",
    bgSecondary: "rgba(30, 41, 59, 0.6)",
    bgTertiary: "rgba(51, 65, 85, 0.5)",
    primary: "#60a5fa",
    secondary: "#818cf8",
    textPrimary: "#f1f5f9",
    textSecondary: "#cbd5e1",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default midnightTheme;
