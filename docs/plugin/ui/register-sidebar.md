---
title: sl.ui.register_sidebar(config)
description: "注册插件侧边栏入口"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.register_sidebar(config)

注册插件侧边栏入口

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/sidebar.rs:6
sidebar::register()
```

## LUA API

### INPUT

- `config`: `table` | 见当前接口的参数约定。
- `label`: `string` | 见当前接口的参数约定。
- Optional `icon`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 注册插件侧边栏入口

### HOW TO USE

注册一个插件侧边栏入口，通常与插件页面或导航配合使用。

```lua
local ok = sl.ui.register_sidebar({ label = "My Plugin" }, "value", "value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
