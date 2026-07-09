---
title: sl.element.set_attribute(selector, attr, value)
description: "批量设置匹配元素的单个属性，`nil` 会被写为空字符串"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.set_attribute(selector, attr, value)

批量设置匹配元素的单个属性，`nil` 会被写为空字符串

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:114
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `attr`: `string` | 见当前接口的参数约定。
- `value`: `string \| number \| boolean \| nil` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 批量设置匹配元素的单个属性，`nil` 会被写为空字符串

### HOW TO USE

批量设置匹配元素的单个属性，`nil` 会被写为空字符串

```lua
local ok = sl.element.set_attribute("#plugin-root", "data-plugin", "example-value")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
