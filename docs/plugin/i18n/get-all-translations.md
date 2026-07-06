---
title: sl.i18n.getAllTranslations()
description: "获取当前语言下可读到的全部翻译项，包含宿主与插件翻译的合并结果"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.getAllTranslations()

获取当前语言下可读到的全部翻译项，包含宿主与插件翻译的合并结果

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/query.rs:73
query::get_all_translations()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `table<string, string>` | 获取当前语言下可读到的全部翻译项，包含宿主与插件翻译的合并结果

### HOW TO USE

获取当前语言下可读到的全部翻译项，包含宿主与插件翻译的合并结果

```lua
local result = sl.i18n.getAllTranslations()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
