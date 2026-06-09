---
title: sl.console.send()
description: "向指定服务器发送一条控制台命令；发送成功返回 `true`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "console"]
author: Codex
---

## sl.console.send()

向指定服务器发送一条控制台命令；发送成功返回 `true`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/console/send.rs:13
send::send()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- `command`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 向指定服务器发送一条控制台命令；发送成功返回 `true`

### HOW TO USE

向指定服务器发送一条控制台命令。

```lua
local ok = sl.console.send("my-server", "say hello from plugin")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/console.md](../../lua-api/console.md)。
