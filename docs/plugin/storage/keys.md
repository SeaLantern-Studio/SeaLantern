---
title: sl.storage.keys()
description: "返回当前所有键名，按字典序排序"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "storage"]
author: Codex
---

## sl.storage.keys()

返回当前所有键名，按字典序排序

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/storage/read.rs:7
register()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `string[]` | 返回当前所有键名，按字典序排序

### HOW TO USE

返回当前所有键名，按字典序排序

```lua
local result = sl.storage.keys()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/storage.md](../../lua-api/storage.md)。
