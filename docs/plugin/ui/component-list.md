---
title: sl.ui.component.list(pageFilter?)
description: "获取当前已镜像的宿主组件列表"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.list(pageFilter?)

获取当前已镜像的宿主组件列表

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:46
component::register_list()
```

## LUA API

### INPUT

- Optional `pageFilter`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table<number, table>` | 获取当前已镜像的宿主组件列表

### HOW TO USE

读取当前已镜像的宿主组件列表，可选按页面过滤。

```lua
local result = sl.ui.component.list("home")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
