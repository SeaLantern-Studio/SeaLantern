---
title: sl.fs.copy(scope, src, dst)
description: "在同一 scope 内复制文件或目录；目标已存在时拒绝覆盖"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.copy(scope, src, dst)

在同一 scope 内复制文件或目录；目标已存在时拒绝覆盖

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/transfer.rs:8
transfer::copy()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `src`: `string` | 见当前接口的参数约定。
- `dst`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 在同一 scope 内复制文件或目录；目标已存在时拒绝覆盖

### HOW TO USE

在同一 scope 内复制文件或目录；目标已存在时拒绝覆盖

```lua
local result = sl.fs.copy("data", "config/settings.json", "backup/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
