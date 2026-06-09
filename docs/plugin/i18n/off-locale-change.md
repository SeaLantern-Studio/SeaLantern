---
title: sl.i18n.offLocaleChange(callbackId)
description: "取消指定 id 的语言切换监听器，成功返回 `true`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.offLocaleChange(callbackId)

取消指定 id 的语言切换监听器，成功返回 `true`

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/events.rs:28
events::off_locale_change()
```

## LUA API

### INPUT

- `callbackId`: `integer` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 取消指定 id 的语言切换监听器，成功返回 `true`

### HOW TO USE

传入之前注册得到的回调 id，成功时返回 `true`。

```lua
local ok = sl.i18n.offLocaleChange(callback_id)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
