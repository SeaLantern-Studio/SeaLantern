---
title: sl.fs.write(scope, path, content)
description: "写入文本内容，不存在的父目录会自动创建"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "fs"]
author: Codex
---

## sl.fs.write(scope, path, content)

写入文本内容，不存在的父目录会自动创建

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/filesystem/write.rs:8
write::write()
```

## LUA API

### INPUT

- `scope`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。
- `content`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `nil` | 写入文本内容，不存在的父目录会自动创建

### HOW TO USE

写入文本内容，不存在的父目录会自动创建

```lua
local result = sl.fs.write("data", "config/settings.json", "enabled=true")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/filesystem.md](../../lua-api/filesystem.md)。
