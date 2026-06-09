---
title: sl.ui.inject_html(elementId, html)
description: "向指定元素注入 HTML 内容"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.inject_html(elementId, html)

向指定元素注入 HTML 内容

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/basic.rs:5
basic::register()
```

## LUA API

### INPUT

- `elementId`: `string` | 见当前接口的参数约定。
- `html`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 向指定元素注入 HTML 内容

### HOW TO USE

向指定元素注入 HTML 内容

```lua
local ok = sl.ui.inject_html("plugin-root", "<div>Hello</div>")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
