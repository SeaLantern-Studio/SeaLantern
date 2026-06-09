---
title: sl.ui.insert(placement, selector, html)
description: "按位置向目标元素前后或内部插入 HTML"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.insert(placement, selector, html)

按位置向目标元素前后或内部插入 HTML

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:154
style::register()
```

## LUA API

### INPUT

- `placement`: `string` | 见当前接口的参数约定。
- `selector`: `string` | 见当前接口的参数约定。
- `html`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 按位置向目标元素前后或内部插入 HTML

### HOW TO USE

按位置向目标元素前后或内部插入 HTML

```lua
local ok = sl.ui.insert("append", "#plugin-root", "<div>Hello</div>")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
