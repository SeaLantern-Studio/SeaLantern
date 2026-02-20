# SeaLantern 主题系统

主题系统支持多种配色方案，包括浅色、深色、浅色亚克力和深色亚克力四种模式。

## 目录结构

```
themes/
├── index.ts      # 主题加载器和管理 API
├── types.ts      # 类型定义
├── default.ts    # 默认主题
├── midnight.ts   # 午夜主题
├── ocean.ts      # 海洋主题
├── rose.ts       # 玫瑰主题
├── sunset.ts     # 日落主题
└── README.md     # 本文档
```

## 快速开始

### 使用主题

```typescript
import { getThemeById, getAllThemes, getThemeColors } from "@/themes";

// 获取所有主题
const themes = getAllThemes();

// 根据 ID 获取主题
const theme = getThemeById("default");

// 获取特定配色方案的颜色
const lightColors = getThemeColors("default", "light");
```

### 注册自定义主题

```typescript
import { registerTheme } from "@/themes";

registerTheme({
  id: "my-theme",
  name: "我的主题",
  light: {
    /* 颜色配置 */
  },
  dark: {
    /* 颜色配置 */
  },
  lightAcrylic: {
    /* 颜色配置 */
  },
  darkAcrylic: {
    /* 颜色配置 */
  },
});
```

## 创建新主题

### 1. 创建主题文件

在 `themes/` 目录下创建新的 `.ts` 文件，例如 `forest.ts`：

```typescript
/**
 * @theme Forest
 * @description 森林主题 - 清新的绿色调
 * @author Your Name
 * @version 1.0.0
 */
import type { ThemeDefinition } from "./types";

const forestTheme: ThemeDefinition = {
  id: "forest",
  name: "Forest",
  description: "森林主题 - 清新的绿色调",
  author: "Your Name",
  version: "1.0.0",
  light: {
    bg: "#f0fdf4",
    bgSecondary: "#dcfce7",
    bgTertiary: "#bbf7d0",
    primary: "#22c55e",
    secondary: "#16a34a",
    textPrimary: "#14532d",
    textSecondary: "#166534",
    border: "#bbf7d0",
  },
  dark: {
    bg: "#0a0f0d",
    bgSecondary: "#141f1a",
    bgTertiary: "#1e2f27",
    primary: "#4ade80",
    secondary: "#22c55e",
    textPrimary: "#ecfdf5",
    textSecondary: "#a7f3d0",
    border: "rgba(255, 255, 255, 0.1)",
  },
  lightAcrylic: {
    bg: "rgba(240, 253, 244, 0.7)",
    bgSecondary: "rgba(220, 252, 231, 0.6)",
    bgTertiary: "rgba(187, 247, 208, 0.5)",
    primary: "#22c55e",
    secondary: "#16a34a",
    textPrimary: "#14532d",
    textSecondary: "#166534",
    border: "#bbf7d0",
  },
  darkAcrylic: {
    bg: "rgba(10, 15, 13, 0.7)",
    bgSecondary: "rgba(20, 31, 26, 0.6)",
    bgTertiary: "rgba(30, 47, 39, 0.5)",
    primary: "#4ade80",
    secondary: "#22c55e",
    textPrimary: "#ecfdf5",
    textSecondary: "#a7f3d0",
    border: "rgba(255, 255, 255, 0.1)",
  },
};

export default forestTheme;
```

### 2. 自动加载

主题系统使用 Vite 的 `import.meta.glob` 自动扫描并加载 `themes/` 目录下的所有主题文件，无需手动注册。

## 类型定义

### ThemeColors

```typescript
interface ThemeColors {
  bg: string; // 主背景色
  bgSecondary: string; // 次级背景色
  bgTertiary: string; // 三级背景色
  primary: string; // 主色调
  secondary: string; // 次级色调
  textPrimary: string; // 主文字颜色
  textSecondary: string; // 次级文字颜色
  border: string; // 边框颜色
}
```

### ThemeDefinition

```typescript
interface ThemeDefinition {
  id: string; // 主题唯一标识符
  name: string; // 主题显示名称
  description?: string; // 主题描述
  author?: string; // 主题作者
  version?: string; // 主题版本
  light: ThemeColors; // 浅色模式配色
  dark: ThemeColors; // 深色模式配色
  lightAcrylic: ThemeColors; // 浅色亚克力模式配色
  darkAcrylic: ThemeColors; // 深色亚克力模式配色
}
```

### ColorPlan

```typescript
type ColorPlan = "light" | "dark" | "lightAcrylic" | "darkAcrylic";
```

## API 参考

| 函数                            | 说明                             |
| ------------------------------- | -------------------------------- |
| `getAllThemes()`                | 获取所有已注册的主题             |
| `getThemeOptions()`             | 获取主题选项列表（用于下拉选择） |
| `getThemeById(id)`              | 根据 ID 获取主题定义             |
| `registerTheme(theme)`          | 注册新主题                       |
| `unregisterTheme(id)`           | 注销主题                         |
| `getThemeColors(themeId, plan)` | 获取主题的配色方案               |
| `resetThemes()`                 | 重置主题系统                     |

## 设计建议

- **亚克力模式**：使用 `rgba()` 格式并设置透明度（推荐 0.5-0.7）
- **深色模式**：确保文字与背景有足够对比度
- **主色调**：建议使用品牌色或符合主题氛围的颜色
