---
title: sl.storage.remove(key)
description: "删除指定键；键不存在时静默忽略"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "storage"]
author: Codex
---

## sl.storage.remove(key)

删除指定键；键不存在时静默忽略

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/storage/write.rs:8
register()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 删除指定键；键不存在时静默忽略

### HOW TO USE

删除指定键；键不存在时静默忽略

```lua
local result = sl.storage.remove("plugins.example.menu.title")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/storage.md](../../lua-api/storage.md)。
