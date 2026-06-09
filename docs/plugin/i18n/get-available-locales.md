---
title: sl.i18n.getAvailableLocales()
description: "获取当前可用语言列表"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.getAvailableLocales()

获取当前可用语言列表

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:81
query::get_available_locales()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `table<number, string>` | 获取当前可用语言列表

### HOW TO USE

获取当前可用语言列表

```lua
local result = sl.i18n.getAvailableLocales()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
