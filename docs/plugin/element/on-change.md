---
title: sl.element.on_change(selector, callback)
description: "监听元素 `change` 事件，返回 cleanup 函数用于解除监听"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.on_change(selector, callback)

监听元素 `change` 事件，返回 cleanup 函数用于解除监听

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/watch.rs:7
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `callback`: `function` | 见当前接口的参数约定。

### OUTPUT

- `result`: `function` | 监听元素 `change` 事件，返回 cleanup 函数用于解除监听

### HOW TO USE

注册 DOM change 监听后，记得在不需要时调用 cleanup 解除监听。

```lua
local cleanup = sl.element.on_change("#plugin-root", function(...) end)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
