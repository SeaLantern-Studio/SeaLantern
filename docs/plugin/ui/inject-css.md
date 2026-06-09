---
title: sl.ui.inject_css(styleId, css)
description: "注入一段具名 CSS"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.inject_css(styleId, css)

注入一段具名 CSS

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:8
style::register()
```

## LUA API

### INPUT

- `styleId`: `string` | 见当前接口的参数约定。
- `css`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 注入一段具名 CSS

### HOW TO USE

注入一段具名 CSS

```lua
local ok = sl.ui.inject_css("plugin-style", ".plugin-panel { color: #42b883; }")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
