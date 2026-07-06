---
title: sl.fs.list(scope, path)
description: "列出目录下的直接子项名称"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.list(scope, path)

列出目录下的直接子项名称

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:78
read::list()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<number, string>` | 列出目录下的直接子项名称

### HOW TO USE

列出目录下的直接子项名称

```lua
local result = sl.fs.list("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
