---
title: sl.server.read_file(serverId, path)
description: "读取服务器目录中的文本文件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.read_file(serverId, path)

读取服务器目录中的文本文件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:31
files::read_file()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 读取服务器目录中的文本文件

### HOW TO USE

读取服务器目录中的文本文件

```lua
local result = sl.server.read_file("my-server", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
