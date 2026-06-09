---
title: sl.element.click(selector)
description: "触发元素点击行为"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.click(selector)

触发元素点击行为

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:39
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 触发元素点击行为

### HOW TO USE

触发元素点击行为

```lua
local ok = sl.element.click("#plugin-root")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
