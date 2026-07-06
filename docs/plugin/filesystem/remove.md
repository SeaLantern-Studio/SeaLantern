---
title: sl.fs.remove(scope, path)
description: "删除文件或空目录；拒绝递归删除非空目录与 sandbox 根"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.remove(scope, path)

删除文件或空目录；拒绝递归删除非空目录与 sandbox 根

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/write.rs:63
write::remove()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 删除文件或空目录；拒绝递归删除非空目录与 sandbox 根

### HOW TO USE

删除文件或空目录；拒绝递归删除非空目录与 sandbox 根

```lua
local result = sl.fs.remove("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
