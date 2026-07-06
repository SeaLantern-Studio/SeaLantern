---
title: sl.server.get_path(serverId)
description: "获取指定服务器的根目录路径"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.get_path(serverId)

获取指定服务器的根目录路径

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:22
files::get_path()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 获取指定服务器的根目录路径

### HOW TO USE

获取指定服务器的根目录路径

```lua
local result = sl.server.get_path("my-server")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
