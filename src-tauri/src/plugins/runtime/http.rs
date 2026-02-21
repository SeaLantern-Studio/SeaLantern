use super::helpers::json_value_from_lua;
use super::PluginRuntime;
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};

fn is_ssrf_url(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return true,
    };

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return true;
    }

    let host = match parsed.host_str() {
        Some(h) => h.to_lowercase(),
        None => return true,
    };

    if host == "::1" || host == "[::1]" {
        return true;
    }

    if host == "localhost" {
        return true;
    }

    if let Ok(addr) = host.parse::<std::net::Ipv4Addr>() {
        let octets = addr.octets();

        if octets[0] == 127 {
            return true;
        }

        if octets[0] == 10 {
            return true;
        }

        if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
            return true;
        }

        if octets[0] == 192 && octets[1] == 168 {
            return true;
        }
    }

    false
}

impl PluginRuntime {
    pub(super) fn setup_http_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;

        let http_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create http table: {}", e))?;

        let plugin_id = self.plugin_id.clone();
        let permissions = self.permissions.clone();

        const MAX_RESPONSE_SIZE: u64 = 5 * 1024 * 1024;

        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let get_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                if !perms.iter().any(|p| p == "network") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'network' permission required",
                    ));
                }
                let url: String = args
                    .front()
                    .and_then(|v| match v {
                        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        mlua::Error::runtime("sl.http.get 第一个参数必须是 URL 字符串")
                    })?;

                let _ = emit_permission_log(&pid, "api_call", "sl.http.get", &url);

                if is_ssrf_url(&url) {
                    return Err(mlua::Error::runtime(
                        "SSRF: 不允许访问内网、本地或非 HTTP(S) 地址",
                    ));
                }

                let (headers, timeout) = parse_http_options(args.get(1))?;

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| mlua::Error::runtime(format!("创建 HTTP 客户端失败: {}", e)))?;

                let mut req = client.get(&url);
                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                match req.send() {
                    Ok(resp) => build_response_table(lua, resp, MAX_RESPONSE_SIZE),
                    Err(e) => {
                        let nil = Value::Nil;
                        let err = Value::String(lua.create_string(format!("{}", e))?);
                        Ok(MultiValue::from_vec(vec![nil, err]))
                    }
                }
            })
            .map_err(|e| format!("Failed to create http.get: {}", e))?;
        http_table
            .set("get", get_fn)
            .map_err(|e| format!("Failed to set http.get: {}", e))?;

        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let post_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                if !perms.iter().any(|p| p == "network") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'network' permission required",
                    ));
                }
                let url: String = args
                    .front()
                    .and_then(|v| match v {
                        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        mlua::Error::runtime("sl.http.post 第一个参数必须是 URL 字符串")
                    })?;

                let _ = emit_permission_log(&pid, "api_call", "sl.http.post", &url);

                if is_ssrf_url(&url) {
                    return Err(mlua::Error::runtime(
                        "SSRF: 不允许访问内网、本地或非 HTTP(S) 地址",
                    ));
                }

                let (body_str, is_json) = lua_body_to_string(args.get(1))?;
                let (mut headers, timeout) = parse_http_options(args.get(2))?;

                if is_json
                    && !headers
                        .iter()
                        .any(|(k, _)| k.to_lowercase() == "content-type")
                {
                    headers.push(("Content-Type".to_string(), "application/json".to_string()));
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| mlua::Error::runtime(format!("创建 HTTP 客户端失败: {}", e)))?;

                let mut req = client.post(&url).body(body_str);
                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                match req.send() {
                    Ok(resp) => build_response_table(lua, resp, MAX_RESPONSE_SIZE),
                    Err(e) => {
                        let nil = Value::Nil;
                        let err = Value::String(lua.create_string(format!("{}", e))?);
                        Ok(MultiValue::from_vec(vec![nil, err]))
                    }
                }
            })
            .map_err(|e| format!("Failed to create http.post: {}", e))?;
        http_table
            .set("post", post_fn)
            .map_err(|e| format!("Failed to set http.post: {}", e))?;

        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let put_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                if !perms.iter().any(|p| p == "network") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'network' permission required",
                    ));
                }
                let url: String = args
                    .front()
                    .and_then(|v| match v {
                        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        mlua::Error::runtime("sl.http.put 第一个参数必须是 URL 字符串")
                    })?;

                let _ = emit_permission_log(&pid, "api_call", "sl.http.put", &url);

                if is_ssrf_url(&url) {
                    return Err(mlua::Error::runtime(
                        "SSRF: 不允许访问内网、本地或非 HTTP(S) 地址",
                    ));
                }

                let (body_str, is_json) = lua_body_to_string(args.get(1))?;
                let (mut headers, timeout) = parse_http_options(args.get(2))?;

                if is_json
                    && !headers
                        .iter()
                        .any(|(k, _)| k.to_lowercase() == "content-type")
                {
                    headers.push(("Content-Type".to_string(), "application/json".to_string()));
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| mlua::Error::runtime(format!("创建 HTTP 客户端失败: {}", e)))?;

                let mut req = client.put(&url).body(body_str);
                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                match req.send() {
                    Ok(resp) => build_response_table(lua, resp, MAX_RESPONSE_SIZE),
                    Err(e) => {
                        let nil = Value::Nil;
                        let err = Value::String(lua.create_string(format!("{}", e))?);
                        Ok(MultiValue::from_vec(vec![nil, err]))
                    }
                }
            })
            .map_err(|e| format!("Failed to create http.put: {}", e))?;
        http_table
            .set("put", put_fn)
            .map_err(|e| format!("Failed to set http.put: {}", e))?;

        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let delete_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                if !perms.iter().any(|p| p == "network") {
                    return Err(mlua::Error::runtime(
                        "Permission denied: 'network' permission required",
                    ));
                }
                let url: String = args
                    .front()
                    .and_then(|v| match v {
                        Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        mlua::Error::runtime("sl.http.delete 第一个参数必须是 URL 字符串")
                    })?;

                let _ = emit_permission_log(&pid, "api_call", "sl.http.delete", &url);

                if is_ssrf_url(&url) {
                    return Err(mlua::Error::runtime(
                        "SSRF: 不允许访问内网、本地或非 HTTP(S) 地址",
                    ));
                }

                let (headers, timeout) = parse_http_options(args.get(1))?;

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| mlua::Error::runtime(format!("创建 HTTP 客户端失败: {}", e)))?;

                let mut req = client.delete(&url);
                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                match req.send() {
                    Ok(resp) => build_response_table(lua, resp, MAX_RESPONSE_SIZE),
                    Err(e) => {
                        let nil = Value::Nil;
                        let err = Value::String(lua.create_string(format!("{}", e))?);
                        Ok(MultiValue::from_vec(vec![nil, err]))
                    }
                }
            })
            .map_err(|e| format!("Failed to create http.delete: {}", e))?;
        http_table
            .set("delete", delete_fn)
            .map_err(|e| format!("Failed to set http.delete: {}", e))?;

        sl.set("http", http_table)
            .map_err(|e| format!("Failed to set sl.http: {}", e))?;

        Ok(())
    }
}

fn parse_http_options(
    options: Option<&Value>,
) -> Result<(Vec<(String, String)>, u64), mlua::Error> {
    let mut headers = Vec::new();
    let mut timeout = 30u64;

    if let Some(Value::Table(opts)) = options {
        if let Ok(Value::Table(h)) = opts.get::<Value>("headers") {
            for (k, v) in h.pairs::<String, String>().flatten() {
                headers.push((k, v));
            }
        }

        if let Ok(t) = opts.get::<u64>("timeout") {
            if t > 0 && t <= 300 {
                timeout = t;
            }
        }
    }

    Ok((headers, timeout))
}

fn lua_body_to_string(body: Option<&Value>) -> Result<(String, bool), mlua::Error> {
    match body {
        Some(Value::String(s)) => {
            let s = s.to_str().map(|s| s.to_string()).unwrap_or_default();
            Ok((s, false))
        }
        Some(Value::Table(_)) => {
            let json_val = json_value_from_lua(body.unwrap(), 0)?;
            let json_str = serde_json::to_string(&json_val)
                .map_err(|e| mlua::Error::runtime(format!("序列化 body 为 JSON 失败: {}", e)))?;
            Ok((json_str, true))
        }
        Some(Value::Nil) | None => Ok((String::new(), false)),
        _ => Err(mlua::Error::runtime("body 参数必须是字符串或 table")),
    }
}

fn build_response_table(
    lua: &Lua,
    resp: reqwest::blocking::Response,
    max_size: u64,
) -> LuaResult<MultiValue> {
    let status = resp.status().as_u16();

    let headers_table = lua.create_table()?;
    for (name, value) in resp.headers().iter() {
        if let Ok(v) = value.to_str() {
            headers_table.set(name.as_str().to_string(), v.to_string())?;
        }
    }

    if let Some(len) = resp.content_length() {
        if len > max_size {
            let nil = Value::Nil;
            let err = Value::String(
                lua.create_string(format!("响应体过大: {} 字节，限制 {} 字节", len, max_size))?,
            );
            return Ok(MultiValue::from_vec(vec![nil, err]));
        }
    }

    let body_bytes = resp
        .bytes()
        .map_err(|e| mlua::Error::runtime(format!("读取响应体失败: {}", e)))?;

    if body_bytes.len() as u64 > max_size {
        let nil = Value::Nil;
        let err = Value::String(lua.create_string(format!(
            "响应体过大: {} 字节，限制 {} 字节",
            body_bytes.len(),
            max_size
        ))?);
        return Ok(MultiValue::from_vec(vec![nil, err]));
    }

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let response_table = lua.create_table()?;
    response_table.set("status", status)?;
    response_table.set("body", body_str)?;
    response_table.set("headers", headers_table)?;

    let resp_val = Value::Table(response_table);
    let nil = Value::Nil;
    Ok(MultiValue::from_vec(vec![resp_val, nil]))
}
