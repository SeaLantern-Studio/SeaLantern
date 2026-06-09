---
title: sl.ui.component.call(componentId, method)
description: "请求调用组件方法"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.component.call(componentId, method)

请求调用组件方法

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/component.rs:119
component::register_call()
```

## LUA API

### INPUT

- `componentId`: `string` | 见当前接口的参数约定。
- `method`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 请求调用组件方法

### HOW TO USE

向宿主请求调用组件方法。

```lua
local ok = sl.ui.component.call("plugin-card", "open")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
