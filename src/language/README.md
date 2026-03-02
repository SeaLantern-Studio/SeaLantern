# SeaLantern 语言系统

国际化系统支持多语言切换，使用 JSON 文件存储翻译文本。

## 目录结构

```
language/
├── index.ts      # i18n 核心模块
├── zh-CN.json    # 简体中文
├── zh-TW.json    # 繁体中文
├── en-US.json    # 英语
├── ja-JP.json    # 日语
├── ko-KR.json    # 韩语
├── de-DE.json    # 德语
├── es-ES.json    # 西班牙语
├── fr-FA.json    # 波斯语
├── ru-RU.json    # 俄语
├── vi-VN.json    # 越南语
├── README.md     # 本文档
└── README-en.md  # 本文档的英文版
```

## 快速开始

### 使用翻译

```typescript
import { i18n } from "@language";

// 获取翻译文本
const text = i18n.t("common.home");

// 带变量的翻译
const message = i18n.t("home.delete_confirm_message", { server: "MyServer" });

// 切换语言
i18n.setLocale("en-US");

// 获取当前语言
const locale = i18n.getLocale();
```

### 在 Vue 组件中使用

```typescript
import { i18n } from "@language";

// 响应式获取当前语言
const currentLocale = i18n.getLocaleRef();

// 翻译函数
const t = (key: string) => i18n.t(key);
```

## 添加新语言

### 1. 创建语言文件

在 `language/` 目录下创建 `语言代码.json` 文件：

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

### 2. 自动加载

语言系统使用 Vite 的 `import.meta.glob` 自动扫描并加载所有 `.json` 文件，无需手动注册。

### 3. 语言代码规范

遵循 ISO 639-1 标准，格式为 `语言-地区`：

| 代码  | 语言     |
| ----- | -------- |
| zh-CN | 简体中文 |
| zh-TW | 繁体中文 |
| en-US | 英语     |
| ja-JP | 日语     |
| ko-KR | 韩语     |
| de-DE | 德语     |
| es-ES | 西班牙语 |
| ru-RU | 俄语     |

## 文件结构

语言文件采用嵌套对象结构：

```typescript
type TranslationNode = {
  [key: string]: string | TranslationNode;
};

type LanguageFile = TranslationNode & {
  languageName?: string; // 语言显示名称
};
```

### 主要模块

| 模块       | 说明                     |
| ---------- | ------------------------ |
| `common`   | 通用文本（按钮、状态等） |
| `sidebar`  | 侧边栏                   |
| `home`     | 首页                     |
| `create`   | 创建服务器               |
| `console`  | 控制台                   |
| `config`   | 配置编辑                 |
| `players`  | 玩家管理                 |
| `settings` | 设置                     |
| `about`    | 关于页面                 |
| `tray`     | 系统托盘                 |

## API 参考

### i18n 实例

| 方法                        | 说明                       |
| --------------------------- | -------------------------- |
| `t(key, options?)`          | 获取翻译文本，支持变量插值 |
| `setLocale(locale)`         | 设置当前语言               |
| `getLocale()`               | 获取当前语言代码           |
| `getLocaleRef()`            | 获取响应式语言引用         |
| `getAvailableLocales()`     | 获取所有支持的语言列表     |
| `isSupportedLocale(locale)` | 检查语言是否支持           |

### 变量插值

支持两种占位符格式：

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

## 最佳实践

1. **保持一致性** - 相同概念使用相同术语
2. **简洁明了** - 避免过长的翻译文本
3. **保留占位符** - `{{variable}}` 和 `{variable}` 不要翻译
4. **测试覆盖** - 添加语言后测试所有页面

## 贡献翻译

1. 复制 `zh-CN.json` 或 `en-US.json` 作为模板
2. 翻译所有文本内容
3. 提交 PR 到 GitHub 仓库

感谢你的贡献！
