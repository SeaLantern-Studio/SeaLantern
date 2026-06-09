---
title: sl.http.delete(url, options?)
description: "发起 `DELETE` 请求"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "http"]
author: Codex
---

## sl.http.delete(url, options?)

发起 `DELETE` 请求

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/http/mod.rs:73
execute_http_request()
```

## LUA API

### INPUT

- `url`: `string` | 见当前接口的参数约定。
- Optional `options`: `table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table | nil, string?` | 发起 `DELETE` 请求

### HOW TO USE

删除请求没有请求体时可直接传 URL。

```lua
local resp, err = sl.http.delete("https://example.com/api", { timeout = 10 })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/http.md](../../lua-api/http.md)。
