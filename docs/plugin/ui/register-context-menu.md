---
title: sl.ui.register_context_menu(context, items)
description: "在指定上下文注册插件菜单项"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.register_context_menu(context, items)

在指定上下文注册插件菜单项

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/context_menu.rs:8
context_menu::register()
```

## LUA API

### INPUT

- `context`: `string` | 见当前接口的参数约定。
- `items`: `table<number, table>` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 在指定上下文注册插件菜单项

### HOW TO USE

为指定上下文注册插件菜单项。

```lua
local ok = sl.ui.register_context_menu("server-list", { { id = "open", label = "Open" } })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
