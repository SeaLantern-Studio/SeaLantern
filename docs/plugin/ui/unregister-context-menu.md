---
title: sl.ui.unregister_context_menu(context)
description: "注销指定上下文下的插件菜单项"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.unregister_context_menu(context)

注销指定上下文下的插件菜单项

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/context_menu.rs:60
context_menu::register()
```

## LUA API

### INPUT

- `context`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 注销指定上下文下的插件菜单项

### HOW TO USE

移除当前插件在指定上下文下注册的菜单项。

```lua
local ok = sl.ui.unregister_context_menu("server-list")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
