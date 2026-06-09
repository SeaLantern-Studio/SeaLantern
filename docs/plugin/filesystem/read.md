---
title: sl.fs.read(scope, path)
description: "读取文本文件内容"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.read(scope, path)

读取文本文件内容

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:9
read::read()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 读取文本文件内容

### HOW TO USE

读取文本文件内容

```lua
local result = sl.fs.read("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
