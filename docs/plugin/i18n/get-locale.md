---
title: sl.i18n.getLocale()
description: "获取当前应用语言代码，例如 `zh-CN`、`en-US`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.getLocale()

获取当前应用语言代码，例如 `zh-CN`、`en-US`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:5
query::get_locale()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `string` | 获取当前应用语言代码，例如 `zh-CN`、`en-US`

### HOW TO USE

获取当前应用语言代码，例如 `zh-CN`、`en-US`

```lua
local result = sl.i18n.getLocale()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
