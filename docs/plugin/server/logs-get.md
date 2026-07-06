---
title: sl.server.logs.get(serverId, count?)
description: "获取指定服务器最近 N 条日志，默认 `100`，最大 `1000`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.logs.get(serverId, count?)

获取指定服务器最近 N 条日志，默认 `100`，最大 `1000`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:19
get()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- Optional `count`: `integer` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<number, string>` | 获取指定服务器最近 N 条日志，默认 `100`，最大 `1000`

### HOW TO USE

读取单台服务器最近的日志内容。

```lua
local result = sl.server.logs.get("my-server", 100)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
