---
title: sl.ui.remove_css(styleId)
description: "移除此前注入的具名 CSS"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.remove_css(styleId)

移除此前注入的具名 CSS

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:34
style::register()
```

## LUA API

### INPUT

- `styleId`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 移除此前注入的具名 CSS

### HOW TO USE

移除此前注入的具名 CSS

```lua
local ok = sl.ui.remove_css("plugin-style")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
