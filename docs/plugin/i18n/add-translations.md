---
title: sl.i18n.addTranslations(locale, entries)
description: "为插件写入指定语言的翻译项；写入时会自动添加插件命名空间前缀"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.addTranslations(locale, entries)

为插件写入指定语言的翻译项；写入时会自动添加插件命名空间前缀

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/write.rs:34
write::add_translations()
```

## LUA API

### INPUT

- `locale`: `string` | 见当前接口的参数约定。
- `entries`: `table<string, string>` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 为插件写入指定语言的翻译项；写入时会自动添加插件命名空间前缀

### HOW TO USE

写入的 key 会自动加上当前插件的命名空间前缀。

```lua
local result = sl.i18n.addTranslations("en-US", { ["menu.title"] = "Plugin Menu" })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
