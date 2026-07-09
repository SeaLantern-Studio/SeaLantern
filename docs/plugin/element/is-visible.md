---
title: sl.element.is_visible(selector)
description: "判断元素是否可见，返回字符串 `\"true\"` / `\"false\"`，超时返回 `nil`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.is_visible(selector)

判断元素是否可见，返回字符串 `"true"` / `"false"`，超时返回 `nil`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/query.rs:82
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string | nil` | 判断元素是否可见，返回字符串 `"true"` / `"false"`，超时返回 `nil`

### HOW TO USE

判断元素是否可见，返回字符串 `"true"` / `"false"`，超时返回 `nil`

```lua
local result = sl.element.is_visible("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
