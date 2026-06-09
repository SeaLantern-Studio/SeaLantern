---
title: sl.ui.component.get(componentId, prop)
description: "请求读取组件属性"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.get(componentId, prop)

请求读取组件属性

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:70
component::register_get()
```

## LUA API

### INPUT

- `componentId`: `string` | 见当前接口的参数约定。
- `prop`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 请求读取组件属性

### HOW TO USE

向宿主请求读取组件属性；这是事件请求型接口，不是同步取值。

```lua
local ok = sl.ui.component.get("plugin-card", "title")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
