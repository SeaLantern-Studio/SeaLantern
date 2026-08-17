import type { ThemeDefinition } from "@type/theme";

// 彩虹主题:静态定义提供背景/文字/边框,主色由 theme.ts 按色相轮询覆盖
export const rainbowTheme: ThemeDefinition = {
  id: "rainbow",
  name: "Rainbow",
  description: "彩虹主题 - 主色按彩虹色相自动轮换",
  author: "SeaLantern Team",
  version: "1.0.0",
  light: {
    bg: "#f8fafc",
    bgSecondary: "#f1f5f9",
    bgTertiary: "#e2e8f0",
    primary: "#ef4444",
    secondary: "#3b82f6",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "#e2e8f0",
  },
  dark: {
    bg: "#0c1222",
    bgSecondary: "#151d2e",
    bgTertiary: "#1e293b",
    primary: "#f87171",
    secondary: "#60a5fa",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.08)",
  },
  lightAcrylic: {
    bg: "rgba(248, 250, 252, 0.65)",
    bgSecondary: "rgba(241, 245, 249, 0.55)",
    bgTertiary: "rgba(226, 232, 240, 0.45)",
    primary: "#ef4444",
    secondary: "#3b82f6",
    textPrimary: "#0f172a",
    textSecondary: "#475569",
    border: "rgba(226, 232, 240, 0.6)",
  },
  darkAcrylic: {
    bg: "rgba(12, 18, 34, 0.65)",
    bgSecondary: "rgba(21, 29, 46, 0.55)",
    bgTertiary: "rgba(30, 41, 59, 0.45)",
    primary: "#f87171",
    secondary: "#60a5fa",
    textPrimary: "#f1f5f9",
    textSecondary: "#94a3b8",
    border: "rgba(255, 255, 255, 0.06)",
  },
};

export default rainbowTheme;
