/**
 * 主题颜色定义
 */
export interface ThemeColors {
  /** 主背景色 */
  bg: string;
  /** 次级背景色 */
  bgSecondary: string;
  /** 三级背景色 */
  bgTertiary: string;
  /** 主色调 */
  primary: string;
  /** 次色调 */
  secondary: string;
  /** 主文本颜色 */
  textPrimary: string;
  /** 次文本颜色 */
  textSecondary: string;
  /** 边框颜色 */
  border: string;
}

/**
 * 颜色方案类型
 */
export type ColorPlan = "light" | "dark" | "lightAcrylic" | "darkAcrylic";

/**
 * 主题定义
 */
export interface ThemeDefinition {
  /** 主题唯一标识 */
  id: string;
  /** 主题显示名称 */
  name: string;
  /** 主题描述 */
  description?: string;
  /** 主题作者 */
  author?: string;
  /** 主题版本 */
  version?: string;
  /** 浅色模式颜色 */
  light: ThemeColors;
  /** 深色模式颜色 */
  dark: ThemeColors;
  /** 浅色亚克力模式颜色 */
  lightAcrylic: ThemeColors;
  /** 深色亚克力模式颜色 */
  darkAcrylic: ThemeColors;
}

/**
 * 主题注册表
 */
export interface ThemeRegistry {
  [id: string]: ThemeDefinition;
}
