---
title: sl.server.list()
description: "列出当前已注册的服务器实例"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.list()

列出当前已注册的服务器实例

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:8
files::list()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `table<number, table>` | 列出当前已注册的服务器实例

### HOW TO USE

列出当前已注册的服务器实例

```lua
local result = sl.server.list()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
