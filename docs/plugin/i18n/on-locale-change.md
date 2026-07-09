---
title: sl.i18n.onLocaleChange(callback)
description: "注册语言切换监听器，返回回调 id，用于后续取消监听"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.onLocaleChange(callback)

注册语言切换监听器，返回回调 id，用于后续取消监听

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/events.rs:7
events::on_locale_change()
```

## LUA API

### INPUT

- `callback`: `function(newLocale: string)` | 见当前接口的参数约定。

### OUTPUT

- `result`: `integer` | 注册语言切换监听器，返回回调 id，用于后续取消监听

### HOW TO USE

保存返回的回调 id，后续可传给 `sl.i18n.offLocaleChange()` 解除监听。

```lua
local callback_id = sl.i18n.onLocaleChange(function(...) end)
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
