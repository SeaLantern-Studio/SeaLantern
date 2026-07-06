---
title: sl.fs.info(scope, path)
description: "获取条目元信息，包含 `size`、`is_dir`、`modified`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.info(scope, path)

获取条目元信息，包含 `size`、`is_dir`、`modified`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:110
read::info()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table` | 获取条目元信息，包含 `size`、`is_dir`、`modified`

### HOW TO USE

获取条目元信息，包含 `size`、`is_dir`、`modified`

```lua
local result = sl.fs.info("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
