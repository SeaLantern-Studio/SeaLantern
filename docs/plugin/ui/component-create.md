---
title: sl.ui.component.create(componentType, componentId, props)
description: "请求创建宿主组件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.create(componentType, componentId, props)

请求创建宿主组件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:167
component::register_create()
```

## LUA API

### INPUT

- `componentType`: `string` | 见当前接口的参数约定。
- `componentId`: `string` | 见当前接口的参数约定。
- `props`: `table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 请求创建宿主组件

### HOW TO USE

向宿主请求创建一个新的镜像组件。

```lua
local ok = sl.ui.component.create("card", "plugin-card", { title = "Hello" })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
