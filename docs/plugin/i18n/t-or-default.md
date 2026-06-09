---
title: sl.i18n.tOrDefault(key, defaultValue, options?)
description: "若 key 存在翻译则返回翻译结果，否则返回默认值"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.tOrDefault(key, defaultValue, options?)

若 key 存在翻译则返回翻译结果，否则返回默认值

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:40
query::t_or_default()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。
- `defaultValue`: `string` | 见当前接口的参数约定。
- Optional `options`: `table<string, string>` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 若 key 存在翻译则返回翻译结果，否则返回默认值

### HOW TO USE

若 key 存在翻译则返回翻译结果，否则返回默认值

```lua
local result = sl.i18n.tOrDefault("plugins.example.menu.title", "Fallback Title", { timeout = 10 })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
