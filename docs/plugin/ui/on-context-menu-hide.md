---
title: sl.ui.on_context_menu_hide(callback)
description: "注册上下文菜单隐藏回调"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.on_context_menu_hide(callback)

注册上下文菜单隐藏回调

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/context_menu.rs:110
context_menu::register()
```

## LUA API

### INPUT

- `callback`: `function` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 注册上下文菜单隐藏回调

### HOW TO USE

注册菜单隐藏回调。

```lua
local ok = sl.ui.on_context_menu_hide(function(...) end)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
