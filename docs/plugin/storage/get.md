---
title: sl.storage.get(key)
description: "按键读取存储值；键不存在时返回 `nil`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "storage"]
author: Codex
---

## sl.storage.get(key)

按键读取存储值；键不存在时返回 `nil`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/storage/read.rs:7
register()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `any | nil` | 按键读取存储值；键不存在时返回 `nil`

### HOW TO USE

按键读取存储值；键不存在时返回 `nil`

```lua
local result = sl.storage.get("plugins.example.menu.title")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/storage.md](../../lua-api/storage.md)。
