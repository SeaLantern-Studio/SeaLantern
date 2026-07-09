---
title: sl.console.getLogs()
description: "按分页方式读取指定服务器控制台日志，返回结构化结果"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "console"]
author: Codex
---

## sl.console.getLogs()

按分页方式读取指定服务器控制台日志，返回结构化结果

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/console/logs.rs:7
logs::get_logs()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- Optional `offset`: `integer` | 见当前接口的参数约定。
- Optional `count`: `integer` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table` | 按分页方式读取指定服务器控制台日志，返回结构化结果

### HOW TO USE

按偏移量分页读取控制台日志，适合做增量轮询。

```lua
local result = sl.console.getLogs("my-server", 0, 100)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/console.md](../../lua-api/console.md)。
