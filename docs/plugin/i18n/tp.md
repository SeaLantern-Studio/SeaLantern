---
title: sl.i18n.tp(pluginId, key)
description: "读取指定插件命名空间下的翻译，相当于查询 `plugins.{pluginId}.{key}`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.tp(pluginId, key)

读取指定插件命名空间下的翻译，相当于查询 `plugins.{pluginId}.{key}`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:90
query::translate_plugin()
```

## LUA API

### INPUT

- `pluginId`: `string` | 见当前接口的参数约定。
- `key`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 读取指定插件命名空间下的翻译，相当于查询 `plugins.{pluginId}.{key}`

### HOW TO USE

读取指定插件命名空间下的翻译，相当于查询 `plugins.{pluginId}.{key}`

```lua
local result = sl.i18n.tp("example-plugin", "plugins.example.menu.title")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
