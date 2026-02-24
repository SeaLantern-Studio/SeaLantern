# Sea Lantern Language System

The internationalization system supports multiple language switching, using JSON files to store translation texts.

## Directory Structure

```
language/
├── index.ts      # i18n core module
├── zh-CN.json    # Simplified Chinese
├── zh-TW.json    # Traditional Chinese
├── en-US.json    # English
├── ja-JP.json    # Japanese
├── ko-KR.json    # Korean
├── de-DE.json    # German
├── es-ES.json    # Spanish
├── fr-FA.json    # Persian
├── ru-RU.json    # Russian
├── vi-VN.json    # Vietnamese
├── README.md     # Simplified Chinese version of this document
└── README-en.md  # This documentation
```

## Quick Start

### Using Translations

```typescript
import { i18n } from "@language";

// Get translated text
const text = i18n.t("common.home");

// Translation with variables
const message = i18n.t("home.delete_confirm_message", { server: "MyServer" });

// Switch language
i18n.setLocale("en-US");

// Get current language
const locale = i18n.getLocale();
```

### Using in Vue Components

```typescript
import { i18n } from "@language";

// Reactively get current language
const currentLocale = i18n.getLocaleRef();

// Translation function
const t = (key: string) => i18n.t(key);
```

## Adding New Languages

### 1. Create Language File

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

### 2. Auto-loading

The language system uses Vite's `import.meta.glob` to automatically scan and load all `.json` files, no manual registration required.

### 3. Language Code Standards

Follows ISO 639-1 standard, format is `language-region`:

| Code  | Language            |
| ----- | ------------------- |
| zh-CN | Simplified Chinese  |
| zh-TW | Traditional Chinese |
| en-US | English             |
| ja-JP | Japanese            |
| ko-KR | Korean              |
| de-DE | German              |
| es-ES | Spanish             |
| ru-RU | Russian             |

## File Structure

Language files use nested object structure:

```typescript
type TranslationNode = {
  [key: string]: string | TranslationNode;
};

type LanguageFile = TranslationNode & {
  languageName?: string; // Language display name
};
```

### Main Modules

| Module     | Description                          |
| ---------- | ------------------------------------ |
| `common`   | Common texts (buttons, status, etc.) |
| `sidebar`  | Sidebar                              |
| `home`     | Home page                            |
| `create`   | Create server                        |
| `console`  | Console                              |
| `config`   | Config editor                        |
| `players`  | Player management                    |
| `settings` | Settings                             |
| `about`    | About page                           |
| `tray`     | System tray                          |

## API Reference

### i18n Instance

| Method                      | Description                                          |
| --------------------------- | ---------------------------------------------------- |
| `t(key, options?)`          | Get translated text, supports variable interpolation |
| `setLocale(locale)`         | Set current language                                 |
| `getLocale()`               | Get current language code                            |
| `getLocaleRef()`            | Get reactive language reference                      |
| `getAvailableLocales()`     | Get list of all supported languages                  |
| `isSupportedLocale(locale)` | Check if language is supported                       |

### Variable Interpolation

Supports two placeholder formats:

```json
{
  "welcome": "Welcome, {{name}}!",
  "count": "Total {count} servers"
}
```

```typescript
i18n.t("welcome", { name: "Player" }); // "Welcome, Player!"
i18n.t("count", { count: 5 }); // "Total 5 servers"
```

## Best Practices

1. **Consistency** - Use same terminology for same concepts
2. **Conciseness** - Avoid overly long translation texts
3. **Preserve placeholders** - `{{variable}}` and `{variable}` should not be translated
4. **Test coverage** - Test all pages after adding a language

## Contributing Translations

1. Copy `zh-CN.json` or `en-US.json` as template
2. Translate all text content
3. Submit PR to GitHub repository

Thank you for your contribution!
