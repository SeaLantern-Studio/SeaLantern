---
title: sl.ui.component.set(componentId, prop, value)
description: "请求设置组件属性"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.set(componentId, prop, value)

请求设置组件属性

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:94
component::register_set()
```

## LUA API

### INPUT

- `componentId`: `string` | 见当前接口的参数约定。
- `prop`: `string` | 见当前接口的参数约定。
- `value`: `any` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 请求设置组件属性

### HOW TO USE

向宿主请求设置组件属性。

```lua
local ok = sl.ui.component.set("plugin-card", "title", "example-value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
