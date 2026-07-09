---
title: sl.server.list_dir(serverId, path)
description: "列出目录下的直接子项及其基础元信息"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.list_dir(serverId, path)

列出目录下的直接子项及其基础元信息

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:75
files::list_dir()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<number, table>` | 列出目录下的直接子项及其基础元信息

### HOW TO USE

列出目录下的直接子项及其基础元信息

```lua
local result = sl.server.list_dir("my-server", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
