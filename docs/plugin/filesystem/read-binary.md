---
title: sl.fs.read_binary(scope, path)
description: "读取二进制文件并返回 Base64 字符串"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.read_binary(scope, path)

读取二进制文件并返回 Base64 字符串

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/read.rs:31
read::read_binary()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `string` | 读取二进制文件并返回 Base64 字符串

### HOW TO USE

读取二进制文件并返回 Base64 字符串

```lua
local result = sl.fs.read_binary("data", "config/settings.json")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
