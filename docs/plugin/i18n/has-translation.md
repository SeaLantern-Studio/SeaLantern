---
title: sl.i18n.hasTranslation(key, locale?)
description: "判断指定 key 是否存在翻译；不传 `locale` 时使用当前语言，带 fallback 行为"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.hasTranslation(key, locale?)

判断指定 key 是否存在翻译；不传 `locale` 时使用当前语言，带 fallback 行为

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:26
query::has_translation()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。
- Optional `locale`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 判断指定 key 是否存在翻译；不传 `locale` 时使用当前语言，带 fallback 行为

### HOW TO USE

判断指定 key 是否存在翻译；不传 `locale` 时使用当前语言，带 fallback 行为

```lua
local ok = sl.i18n.hasTranslation("plugins.example.menu.title", "en-US")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
