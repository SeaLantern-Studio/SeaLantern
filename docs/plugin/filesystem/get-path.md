---
title: sl.fs.get_path(scope)
description: "获取虚拟 sandbox 路径标识，而非宿主真实路径"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.get_path(scope)

获取虚拟 sandbox 路径标识，而非宿主真实路径

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:144
read::get_path()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 获取虚拟 sandbox 路径标识，而非宿主真实路径

### HOW TO USE

获取虚拟 sandbox 路径标识，而非宿主真实路径

```lua
local result = sl.fs.get_path("data")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
