---
title: sl.element.set_style(selector, styles)
description: "批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.set_style(selector, styles)

批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:141
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `styles`: `table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端

### HOW TO USE

批量设置匹配元素样式，支持 Lua table 转 JSON 后下发前端

```lua
local ok = sl.element.set_style("#plugin-root", { color = "#42b883" })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
