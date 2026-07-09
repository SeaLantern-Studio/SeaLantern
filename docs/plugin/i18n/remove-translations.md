---
title: sl.i18n.removeTranslations()
description: "移除当前插件此前注册的全部翻译项"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "i18n"]
author: Codex
---

## sl.i18n.removeTranslations()

移除当前插件此前注册的全部翻译项

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/i18n/write.rs:53
write::remove_translations()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `nil` | 移除当前插件此前注册的全部翻译项

### HOW TO USE

用于清理当前插件此前注册过的翻译资源。

```lua
local result = sl.i18n.removeTranslations()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/i18n.md](../../lua-api/i18n.md)。
