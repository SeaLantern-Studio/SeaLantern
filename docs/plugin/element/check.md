---
title: sl.element.check(selector, checked)
description: "设置复选框 / 单选框选中状态，并派发 `change` 事件"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.check(selector, checked)

设置复选框 / 单选框选中状态，并派发 `change` 事件

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:39
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `checked`: `boolean` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 设置复选框 / 单选框选中状态，并派发 `change` 事件

### HOW TO USE

设置复选框 / 单选框选中状态，并派发 `change` 事件

```lua
local ok = sl.element.check("#plugin-root", true)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
