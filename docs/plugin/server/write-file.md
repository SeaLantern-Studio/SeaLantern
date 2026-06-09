---
title: sl.server.write_file(serverId, path, content)
description: "向服务器目录写入文本文件，不存在的父目录会自动创建"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "server"]
author: Codex
---

## sl.server.write_file(serverId, path, content)

向服务器目录写入文本文件，不存在的父目录会自动创建

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/server/files.rs:49
files::write_file()
```

## LUA API

### INPUT

- `serverId`: `string` | 见当前接口的参数约定。
- `path`: `string` | 见当前接口的参数约定。
- `content`: `string` | 见当前接口的参数约定。

### OUTPUT

- `result`: `boolean` | 向服务器目录写入文本文件，不存在的父目录会自动创建

### HOW TO USE

向服务器目录写入文本文件，不存在的父目录会自动创建

```lua
local ok = sl.server.write_file("my-server", "config/settings.json", "enabled=true")
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/server.md](../../lua-api/server.md)。
