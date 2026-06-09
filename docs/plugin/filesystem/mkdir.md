---
title: sl.fs.mkdir(scope, path)
description: "创建目录，等价于递归创建"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.mkdir(scope, path)

创建目录，等价于递归创建

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/write.rs:38
write::mkdir()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 创建目录，等价于递归创建

### HOW TO USE

创建目录，等价于递归创建

```lua
local result = sl.fs.mkdir("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
