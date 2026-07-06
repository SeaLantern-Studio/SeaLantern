---
title: sl.ui.set_attribute(selector, attr, value)
description: "设置元素属性"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.set_attribute(selector, attr, value)

设置元素属性

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/style.rs:246
style::register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `attr`: `string` | 见当前接口的参数约定。
- `value`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 设置元素属性

### HOW TO USE

设置元素属性

```lua
local ok = sl.ui.set_attribute("#plugin-root", "data-plugin", "example-value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
