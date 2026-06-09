---
title: sl.ui.unregister_sidebar()
description: "注销当前插件侧边栏入口"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "ui"]
author: Codex
---

## sl.ui.unregister_sidebar()

注销当前插件侧边栏入口

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/ui/sidebar.rs:28
sidebar::register()
```

## LUA API

### INPUT

- 无输入参数。

### OUTPUT

- `result`: `boolean` | 注销当前插件侧边栏入口

### HOW TO USE

注销当前插件此前注册的侧边栏入口。

```lua
local ok = sl.ui.unregister_sidebar()
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/ui.md](../../lua-api/ui.md)。
