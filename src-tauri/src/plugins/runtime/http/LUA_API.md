# http Lua 接口说明

本文档说明插件运行时暴露的 [`sl.http`](../http/mod.rs) Lua 接口，用于在插件中发起受限的 HTTP 网络请求访问外部互联网资源。

## 接口总览

| Lua 接口                             | 参数                                                       | 返回值                  | 说明                                                           | 对应实现                                |
| ------------------------------------ | ---------------------------------------------------------- | ----------------------- | -------------------------------------------------------------- | --------------------------------------- |
| `sl.http.get(url, options?)`         | `url: string`，`options?: table`                           | `table \| nil, string?` | 发起 `GET` 请求，请求成功时返回响应表，失败时返回 `nil, error` | [`execute_http_request()`](./mod.rs:73) |
| `sl.http.post(url, body?, options?)` | `url: string`，`body?: string \| table`，`options?: table` | `table \| nil, string?` | 发起 `POST` 请求，`body` 为 Lua 表时会自动序列化为 JSON        | [`execute_http_request()`](./mod.rs:73) |
| `sl.http.put(url, body?, options?)`  | `url: string`，`body?: string \| table`，`options?: table` | `table \| nil, string?` | 发起 `PUT` 请求，`body` 为 Lua 表时会自动序列化为 JSON         | [`execute_http_request()`](./mod.rs:73) |
| `sl.http.delete(url, options?)`      | `url: string`，`options?: table`                           | `table \| nil, string?` | 发起 `DELETE` 请求                                             | [`execute_http_request()`](./mod.rs:73) |

## 使用说明

### 1. 发起 GET 请求

```lua
local resp, err = sl.http.get("https://httpbin.org/get", {
  timeout = 10,
  headers = {
    ["User-Agent"] = "SeaLanternPlugin/1.0"
  }
})

if not resp then
  print("request failed:", err)
  return
end

print(resp.status)
print(resp.body)
```

### 2. 发起 POST JSON 请求

```lua
local resp, err = sl.http.post("https://httpbin.org/post", {
  name = "SeaLantern",
  enabled = true
}, {
  timeout = 15
})

if not resp then
  print(err)
  return
end

print(resp.status)
print(resp.headers["content-type"] or resp.headers["Content-Type"])
print(resp.body)
```

当 `body` 是 Lua table 时，会在 [`lua_body_to_string()`](./request.rs:107) 中自动序列化成 JSON，并在未显式提供 `Content-Type` 时由 [`with_json_content_type()`](./request.rs:120) 自动补上 `application/json`。

### 3. 发起原始字符串请求体

```lua
local resp, err = sl.http.put("https://httpbin.org/put", "plain text body", {
  headers = {
    ["Content-Type"] = "text/plain"
  }
})

if resp then
  print(resp.body)
end
```

### 4. 发起 DELETE 请求

```lua
local resp, err = sl.http.delete("https://httpbin.org/delete")
if not resp then
  print(err)
  return
end

print(resp.status)
```

## 参数说明

### `url`

- 类型：`string`
- 必填
- 必须是合法的 `http` 或 `https` URL，校验逻辑见 [`extract_url()`](./request.rs:36) 与 [`is_ssrf_url()`](./mod.rs:21)

### `body`

- 类型：`string | table | nil`
- 仅 [`sl.http.post()`](./mod.rs:130) 和 [`sl.http.put()`](./mod.rs:131) 支持
- 当为 `table` 时会被序列化为 JSON
- 当为 `string` 时会原样作为请求体发送

### `options`

`options` 是一个 Lua table，可包含以下字段：

| 字段名    | 类型                    | 默认值 | 说明                               |
| --------- | ----------------------- | ------ | ---------------------------------- |
| `headers` | `table<string, string>` | 空表   | 请求头集合                         |
| `timeout` | `integer`               | `30`   | 请求超时秒数，最小 `1`，最大 `300` |

解析逻辑见 [`parse_http_options()`](./request.rs:58)。若 `options`、`headers` 或 `timeout` 类型错误，接口会直接抛出 Lua runtime error。

## 返回值说明

接口采用 `result, err` 风格返回：

- 成功时返回：`responseTable, nil`
- 请求发送失败时返回：`nil, errorMessage`
- 参数错误、权限不足、安全策略拒绝时：直接抛出 Lua runtime error

成功返回的响应表由 [`build_response_table()`](./response.rs:22) 构造，包含以下字段：

| 字段名    | 类型                    | 说明                                  |
| --------- | ----------------------- | ------------------------------------- |
| `status`  | `integer`               | HTTP 状态码                           |
| `body`    | `string`                | 响应体内容，按 UTF-8 lossy 字符串返回 |
| `headers` | `table<string, string>` | 响应头表                              |

## 权限模型

所有 [`sl.http`](../http/mod.rs) 接口都要求插件拥有 `network` 权限，校验逻辑见 [`execute_http_request()`](./mod.rs:79)。

- 缺少 `network` 权限时会拒绝执行
- 权限上下文由 [`HttpContext`](./common.rs:6) 持有
- 每次调用会通过 [`emit_permission_log()`](../../api.rs:128) 记录权限访问日志

## 安全限制

当前 [`sl.http`](../http/mod.rs) 带有基础的网络访问限制，主要包括：

| 限制项         | 规则                                                               | 对应实现                                   |
| -------------- | ------------------------------------------------------------------ | ------------------------------------------ |
| 协议限制       | 仅允许 `http` 与 `https`                                           | [`is_ssrf_url()`](./mod.rs:21)             |
| 本地/内网限制  | 禁止访问 `localhost`、回环地址、私网地址、链路本地地址、未指定地址 | [`is_private_ip()`](./mod.rs:47)           |
| 重定向限制     | 禁止自动跟随 HTTP 重定向，避免跳转绕过 SSRF 规则                   | [`create_http_client()`](./mod.rs:65)      |
| 超时限制       | 默认 `30s`，允许范围 `1s` 到 `300s`                                | [`parse_http_options()`](./request.rs:58)  |
| 响应体大小限制 | 响应体最大 `5 MiB`，超限时流式读取过程中立即终止                   | [`read_response_body()`](./response.rs:47) |

## 备注

- [`sl.http`](../http/mod.rs) 确实已经对 Lua 暴露 API，注册逻辑见 [`setup_http_namespace()`](./mod.rs:124)。
- Lua 可以通过该模块访问外部互联网资源，但前提是插件拥有 `network` 权限，且目标地址不属于本地或内网受限范围。
- 当前响应体统一按字符串返回，不适合直接处理二进制内容，转换逻辑见 [`build_response_table()`](./response.rs:22)。
- 当前不会自动跟随 3xx 重定向；如目标站点依赖跳转，Lua 侧需要自行处理返回结果。
