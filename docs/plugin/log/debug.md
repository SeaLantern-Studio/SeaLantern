---
title: sl.log.debug()
description: "输出调试日志；当插件缺少日志权限时，该调用会静默忽略"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "log"]
author: Codex
---

## sl.log.debug()

输出调试日志；当插件缺少日志权限时，该调用会静默忽略

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/log/emit.rs:4
create_log_function()
```

## LUA API

### INPUT

- `message`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 输出调试日志；当插件缺少日志权限时，该调用会静默忽略

### HOW TO USE

输出调试日志；当插件缺少日志权限时，该调用会静默忽略

```lua
local result = sl.log.debug("plugin initialized")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/log.md](../../lua-api/log.md)。
