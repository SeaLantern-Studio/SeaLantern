---
title: sl.ui.component.on(componentId, event)
description: "请求订阅组件事件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.on(componentId, event)

请求订阅组件事件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:143
component::register_on()
```

## LUA API

### INPUT

- `componentId`: `string` | 见当前接口的参数约定。
- `event`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 请求订阅组件事件

### HOW TO USE

向宿主请求订阅组件事件。

```lua
local ok = sl.ui.component.on("plugin-card", "confirm")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
