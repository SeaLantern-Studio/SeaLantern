---
title: sl.ui.set_style(selector, styles)
description: "批量设置内联样式"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.set_style(selector, styles)

批量设置内联样式

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:214
style::register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `styles`: `table<string, string>` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 批量设置内联样式

### HOW TO USE

批量设置内联样式

```lua
local ok = sl.ui.set_style("#plugin-root", { color = "#42b883" })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
