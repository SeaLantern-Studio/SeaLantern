---
title: sl.server.exists(serverId, path)
description: "判断服务器目录中的文件或目录是否存在"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.exists(serverId, path)

判断服务器目录中的文件或目录是否存在

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:120
files::exists()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 判断服务器目录中的文件或目录是否存在

### HOW TO USE

判断服务器目录中的文件或目录是否存在

```lua
local ok = sl.server.exists("my-server", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
