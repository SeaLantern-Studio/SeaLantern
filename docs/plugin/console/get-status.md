---
title: sl.console.getStatus()
description: "获取指定服务器当前状态，例如 `running`、`stopped` 等"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "console"]
author: Codex
---

## sl.console.getStatus()

获取指定服务器当前状态，例如 `running`、`stopped` 等

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/console/status.rs:5
status::get_status()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 获取指定服务器当前状态，例如 `running`、`stopped` 等

### HOW TO USE

读取指定服务器当前状态，再决定是否继续发送命令。

```lua
local result = sl.console.getStatus("my-server")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/console.md](../../lua-api/console.md)。
