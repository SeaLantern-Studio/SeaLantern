---
title: sl.ui.set_error_mode(mode)
description: "设置 UI 接口错误模式，支持 `compat` 与 `strict`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.set_error_mode(mode)

设置 UI 接口错误模式，支持 `compat` 与 `strict`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/config.rs:4
config::register()
```

## LUA API

### INPUT

- `mode`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 设置 UI 接口错误模式，支持 `compat` 与 `strict`

### HOW TO USE

设置 UI 接口错误模式，支持 `compat` 与 `strict`

```lua
local ok = sl.ui.set_error_mode("compat")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
