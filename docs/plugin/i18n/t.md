---
title: sl.i18n.t(key, options?)
description: "按 key 获取翻译文本；当提供 `options` 时会进行 `{name}` 风格变量替换"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.t(key, options?)

按 key 获取翻译文本；当提供 `options` 时会进行 `{name}` 风格变量替换

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:9
query::translate()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。
- Optional `options`: `table<string, string>` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 按 key 获取翻译文本；当提供 `options` 时会进行 `{name}` 风格变量替换

### HOW TO USE

按 key 获取翻译文本；当提供 `options` 时会进行 `{name}` 风格变量替换

```lua
local result = sl.i18n.t("plugins.example.menu.title", { timeout = 10 })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
