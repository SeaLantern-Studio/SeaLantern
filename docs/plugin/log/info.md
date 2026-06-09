---
title: sl.log.info()
description: "输出信息日志，并发送对应日志事件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "log"]
author: Codex
---

## sl.log.info()

输出信息日志，并发送对应日志事件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/log/emit.rs:4
create_log_function()
```

## LUA API

### INPUT

- `message`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 输出信息日志，并发送对应日志事件

### HOW TO USE

输出信息日志，并发送对应日志事件

```lua
local result = sl.log.info("plugin initialized")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/log.md](../../lua-api/log.md)。
