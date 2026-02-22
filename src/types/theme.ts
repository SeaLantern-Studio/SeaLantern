export type ColorPlan = "light" | "dark" | "lightAcrylic" | "darkAcrylic";

export interface ThemeColors {
  bg: string;
  bgSecondary: string;
  bgTertiary: string;
  primary: string;
  secondary: string;
  textPrimary: string;
  textSecondary: string;
  border: string;
  [key: string]: string;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  description?: string;
  author?: string;
  version?: string;
  light: ThemeColors;
  dark: ThemeColors;
  lightAcrylic: ThemeColors;
  darkAcrylic: ThemeColors;
  [key: string]: unknown;
}

export type ThemeRegistry = Record<string, ThemeDefinition>;
