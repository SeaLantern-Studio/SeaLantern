---
title: sl.i18n.registerLocale(locale, displayName)
description: "注册插件提供的新语言及其显示名称"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.registerLocale(locale, displayName)

注册插件提供的新语言及其显示名称

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/write.rs:16
write::register_locale()
```

## LUA API

### INPUT

- `locale`: `string` | 见当前接口的参数约定。
- `displayName`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 注册插件提供的新语言及其显示名称

### HOW TO USE

先注册语言，再写入该语言的翻译条目。

```lua
local result = sl.i18n.registerLocale("en-US", "English (US)")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
