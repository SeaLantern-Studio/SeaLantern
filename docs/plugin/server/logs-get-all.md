---
title: sl.server.logs.getAll(count?)
description: "获取所有运行中服务器最近 N 条日志，默认 `100`，最大 `1000`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.logs.getAll(count?)

获取所有运行中服务器最近 N 条日志，默认 `100`，最大 `1000`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/logs.rs:37
get_all()
```

## LUA API

### INPUT

- Optional `count`: `integer` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<number, table>` | 获取所有运行中服务器最近 N 条日志，默认 `100`，最大 `1000`

### HOW TO USE

批量读取所有运行中服务器的最近日志。

```lua
local result = sl.server.logs.getAll(100)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
