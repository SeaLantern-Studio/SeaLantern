---
title: sl.element.set_value(selector, value)
description: "设置输入类元素的值，并派发 `input` / `change` 事件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.set_value(selector, value)

设置输入类元素的值，并派发 `input` / `change` 事件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:39
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `value`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 设置输入类元素的值，并派发 `input` / `change` 事件

### HOW TO USE

设置输入类元素的值，并派发 `input` / `change` 事件

```lua
local ok = sl.element.set_value("#plugin-root", "example-value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
