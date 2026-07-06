---
title: sl.fs.rename(scope, oldPath, newPath)
description: "在同一 scope 内重命名文件或目录"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.rename(scope, oldPath, newPath)

在同一 scope 内重命名文件或目录

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/transfer.rs:77
transfer::rename_entry()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `oldPath`: `string` | 见当前接口的参数约定。
- `newPath`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 在同一 scope 内重命名文件或目录

### HOW TO USE

在同一 scope 内重命名文件或目录

```lua
local result = sl.fs.rename("data", "backup/settings.json", "backup/settings.old.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
