---
title: sl.ui.enable(selector)
description: "启用匹配选择器的元素"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.enable(selector)

启用匹配选择器的元素

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:130
style::register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 启用匹配选择器的元素

### HOW TO USE

启用匹配选择器的元素

```lua
local ok = sl.ui.enable("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
