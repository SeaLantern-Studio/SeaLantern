---
title: sl.ui.toast(type, message, duration?)
description: "显示一条 Toast 提示"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.toast(type, message, duration?)

显示一条 Toast 提示

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/feedback.rs:6
feedback::register()
```

## LUA API

### INPUT

- `type`: `string` | 见当前接口的参数约定。
- `message`: `string` | 见当前接口的参数约定。
- Optional `duration`: `integer` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 显示一条 Toast 提示

### HOW TO USE

显示一条 Toast 提示

```lua
local ok = sl.ui.toast("success", "plugin initialized", 3000)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
