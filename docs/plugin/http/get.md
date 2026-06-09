---
title: sl.http.get(url, options?)
description: "发起 `GET` 请求，请求成功时返回响应表，失败时返回 `nil, error`"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "http"]
author: Codex
---

## sl.http.get(url, options?)

发起 `GET` 请求，请求成功时返回响应表，失败时返回 `nil, error`

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

- `result`: `table | nil, string?` | 发起 `GET` 请求，请求成功时返回响应表，失败时返回 `nil, error`

### HOW TO USE

请求成功时返回响应表，失败时返回 `nil, error`。

```lua
local resp, err = sl.http.get("https://example.com/api", { timeout = 10 })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/http.md](../../lua-api/http.md)。
