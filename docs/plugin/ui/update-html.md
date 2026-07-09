---
title: sl.ui.update_html(elementId, html)
description: "更新指定元素的 HTML 内容"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.update_html(elementId, html)

更新指定元素的 HTML 内容

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/basic.rs:57
basic::register()
```

## LUA API

### INPUT

- `elementId`: `string` | 见当前接口的参数约定。
- `html`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 更新指定元素的 HTML 内容

### HOW TO USE

更新指定元素的 HTML 内容

```lua
local ok = sl.ui.update_html("plugin-root", "<div>Hello</div>")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
