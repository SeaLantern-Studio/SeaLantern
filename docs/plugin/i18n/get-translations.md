---
title: sl.i18n.getTranslations(locale)
description: "获取指定语言下可读到的全部翻译项"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.getTranslations(locale)

获取指定语言下可读到的全部翻译项

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:77
query::get_translations()
```

## LUA API

### INPUT

- `locale`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<string, string>` | 获取指定语言下可读到的全部翻译项

### HOW TO USE

获取指定语言下可读到的全部翻译项

```lua
local result = sl.i18n.getTranslations("en-US")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
