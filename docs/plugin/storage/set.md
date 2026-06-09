---
title: sl.storage.set(key, value)
description: "写入或覆盖一个键值；值会先转换为 JSON 再落盘"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "storage"]
author: Codex
---

## sl.storage.set(key, value)

写入或覆盖一个键值；值会先转换为 JSON 再落盘

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/storage/write.rs:8
register()
```

## LUA API

### INPUT

- `key`: `string` | 见当前接口的参数约定。
- `value`: `nil \| boolean \| number \| string \| table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 写入或覆盖一个键值；值会先转换为 JSON 再落盘

### HOW TO USE

写入或覆盖一个键值；值会先转换为 JSON 再落盘

```lua
local result = sl.storage.set("plugins.example.menu.title", "example-value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/storage.md](../../lua-api/storage.md)。
