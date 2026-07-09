---
title: sl.ui.query(selector)
description: "向宿主发起 UI 查询请求"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.query(selector)

向宿主发起 UI 查询请求

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/basic.rs:83
basic::register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 向宿主发起 UI 查询请求

### HOW TO USE

向宿主发起 UI 查询请求

```lua
local ok = sl.ui.query("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
