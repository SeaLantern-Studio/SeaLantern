---
title: sl.element.form_fill(selector, fields)
description: "按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "element"]
author: Codex
---

## sl.element.form_fill(selector, fields)

按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/element/mutate.rs:162
register()
```

## LUA API

### INPUT

- `selector`: `string` | 见当前接口的参数约定。
- `fields`: `table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组

### HOW TO USE

按字段名批量填写表单，支持 `input` / `textarea` / `select` / 复选框组

```lua
local ok = sl.element.form_fill("#plugin-root", { name = "Sea Lantern" })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/element.md](../../lua-api/element.md)。
