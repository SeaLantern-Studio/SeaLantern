# SeaLantern 语言系统 / Language System

国际化系统支持多语言切换，使用 JSON 文件存储翻译文本。
The internationalization system supports multiple language switching, using JSON files to store translation texts.

## 目录结构 / Directory Structure

```
language/
├── index.ts      # i18n 核心模块 / i18n core module
├── zh-CN.json    # 简体中文 / Simplified Chinese
├── zh-TW.json    # 繁体中文 / Traditional Chinese
├── en-US.json    # 英语 / English
├── ja-JP.json    # 日语 / Japanese
├── ko-KR.json    # 韩语 / Korean
├── de-DE.json    # 德语 / German
├── es-ES.json    # 西班牙语 / Spanish
├── fr-FA.json    # 法语 / French
├── ru-RU.json    # 俄语 / Russian
├── vi-VN.json    # 越南语 / Vietnamese
└── README.md     # 本文档 / This document (bilingual)
```

## 快速开始 / Quick Start

### 使用翻译 / Using Translations

```typescript
import { i18n } from "@language";

// 获取翻译文本 / Get translated text
const text = i18n.t("common.home");

// 带变量的翻译 / Translation with variables
const message = i18n.t("home.delete_confirm_message", { server: "MyServer" });

// 切换语言 / Switch language
i18n.setLocale("en-US");

// 获取当前语言 / Get current language
const locale = i18n.getLocale();
```

### 在 Vue 组件中使用 / Using in Vue Components

```typescript
import { i18n } from "@language";

// 响应式获取当前语言
const currentLocale = i18n.getLocaleRef();

// 翻译函数 / Translation function
const t = (key: string) => i18n.t(key);
```

## 添加新语言 / Adding New Languages

### 1. 创建语言文件 / Create Language File

在 `language/` 目录下创建 `语言代码.json` 文件：
Create a `language-code.json` file in the `language/` directory:

```json
{
  "languageName": "English",
  "common": {
    "app_name": "Sea Lantern",
    "home": "Home",
    "settings": "Settings"
  },
  "home": {
    "title": "Server Management",
    "start": "Start",
    "stop": "Stop"
  }
}
```

### 2. 自动加载 / Auto-loading

语言系统使用 Vite 的 `import.meta.glob` 自动扫描并加载所有 `.json` 文件，无需手动注册。
The language system uses Vite's `import.meta.glob` to automatically scan and load all `.json` files, no manual registration required.

### 3. 语言代码规范 / Language Code Standards

遵循 ISO 639-1 标准，格式为 `语言-地区`：
Follows ISO 639-1 standard, format is `language-region`:

| 代码 / Code | 语言 / Language                |
| ----------- | ------------------------------ |
| zh-CN       | 简体中文 / Simplified Chinese  |
| zh-TW       | 繁体中文 / Traditional Chinese |
| en-US       | 英语 / English                 |
| ja-JP       | 日语 / Japanese                |
| ko-KR       | 韩语 / Korean                  |
| de-DE       | 德语 / German                  |
| es-ES       | 西班牙语 / Spanish             |
| fr-FA       | 法语 / French                  |
| ru-RU       | 俄语 / Russian                 |
| vi-VN       | 越南语 / Vietnamese            |

## 文件结构 / File Structure

语言文件采用嵌套对象结构：
Language files use nested object structure:

```typescript
type TranslationNode = {
  [key: string]: string | TranslationNode;
};

type LanguageFile = TranslationNode & {
  languageName?: string; // 语言显示名称 / Language display name
};
```

### 主要模块 / Main Modules

| 模块 / Module | 说明 / Description                      |
| ------------- | --------------------------------------- |
| `common`      | 通用文本（按钮、状态等） / Common texts |
| `sidebar`     | 侧边栏 / Sidebar                        |
| `home`        | 首页 / Home page                        |
| `create`      | 创建服务器 / Create server              |
| `console`     | 控制台 / Console                        |
| `config`      | 配置编辑 / Config editor                |
| `players`     | 玩家管理 / Player management            |
| `settings`    | 设置 / Settings                         |
| `about`       | 关于页面 / About page                   |
| `tray`        | 系统托盘 / System tray                  |

## API 参考 / API Reference

### i18n 实例 / i18n Instance

| 方法 / Method               | 说明 / Description                                         |
| --------------------------- | ---------------------------------------------------------- |
| `t(key, options?)`          | 获取翻译文本，支持变量插值 / Get translated text           |
| `te(key)`                   | 检查翻译键是否存在 / Check if translation key exists       |
| `setLocale(locale)`         | 设置当前语言 / Set current language                        |
| `getLocale()`               | 获取当前语言代码 / Get current language code               |
| `getLocaleRef()`            | 获取响应式语言引用 / Get reactive language reference       |
| `getAvailableLocales()`     | 获取所有支持的语言列表 / Get list of all supported locales |
| `isSupportedLocale(locale)` | 检查语言是否支持 / Check if locale is supported            |
| `getTranslations()`         | 获取所有翻译数据 / Get all translation data                |

### 变量插值 / Variable Interpolation

支持两种占位符格式：
Supports two placeholder formats:

```json
{
  "welcome": "欢迎, {{name}}!",
  "count": "共 {count} 个服务器"
}
```

```typescript
i18n.t("welcome", { name: "Player" }); // "欢迎, Player!"
i18n.t("count", { count: 5 }); // "共 5 个服务器"
```

## 最佳实践 / Best Practices

1. **保持一致性** / **Consistency** - 相同概念使用相同术语 / Use same terminology for same concepts
2. **简洁明了** / **Conciseness** - 避免过长的翻译文本 / Avoid overly long translation texts
3. **保留占位符** / **Preserve placeholders** - `{{variable}}` 和 `{variable}` 不要翻译 / should not be translated
4. **测试覆盖** / **Test coverage** - 添加语言后测试所有页面 / Test all pages after adding a language

## 贡献翻译 / Contributing Translations

1. 复制 `zh-CN.json` 或 `en-US.json` 作为模板 / Copy as template
2. 翻译所有文本内容 / Translate all text content
3. 提交 PR 到 GitHub 仓库 / Submit PR to GitHub repository

感谢你的贡献 / Thank you for your contribution!
