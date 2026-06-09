---
title: sl.element.get_text(selector)
description: "获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.get_text(selector)

获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/query.rs:82
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string | nil` | 获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`

### HOW TO USE

获取匹配元素的文本内容；查询失败、超时或未找到时返回 `nil`

```lua
local result = sl.element.get_text("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
