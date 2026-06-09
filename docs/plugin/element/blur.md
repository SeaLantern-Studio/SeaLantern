---
title: sl.element.blur(selector)
description: "让元素失去焦点"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.blur(selector)

让元素失去焦点

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:200
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 让元素失去焦点

### HOW TO USE

让元素失去焦点

```lua
local ok = sl.element.blur("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
