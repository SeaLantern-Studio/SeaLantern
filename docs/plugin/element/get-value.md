---
title: sl.element.get_value(selector)
description: "获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.get_value(selector)

获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/query.rs:82
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string | nil` | 获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`

### HOW TO USE

获取输入类元素的当前值；查询失败、超时或未找到时返回 `nil`

```lua
local result = sl.element.get_value("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
