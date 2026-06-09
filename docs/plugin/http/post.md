---
title: sl.http.post(url, body?, options?)
description: "发起 `POST` 请求，`body` 为 Lua 表时会自动序列化为 JSON"
lastUpdated: 2026-06-09
tags: ["plugin", "lua-api", "http"]
author: Codex
---

## sl.http.post(url, body?, options?)

发起 `POST` 请求，`body` 为 Lua 表时会自动序列化为 JSON

## Impl

```rust
// ../../../backend/tauri-host/src/plugins/runtime/http/mod.rs:73
execute_http_request()
```

## LUA API

### INPUT

- `url`: `string` | 见当前接口的参数约定。
- Optional `body`: `string \| table` | 见当前接口的参数约定。
- Optional `options`: `table` | 见当前接口的参数约定。

### OUTPUT

- `result`: `table | nil, string?` | 发起 `POST` 请求，`body` 为 Lua 表时会自动序列化为 JSON

### HOW TO USE

当 `body` 是 Lua table 时会自动序列化为 JSON。

```lua
local resp, err = sl.http.post("https://example.com/api", { name = "SeaLantern" }, { timeout = 10 })
```

## NOTE

- 共享权限、限制和错误语义见 [README](./README.md)。

- 原始模块文档见 [../../lua-api/http.md](../../lua-api/http.md)。
