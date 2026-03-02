use super::helpers::json_value_from_lua;
use super::PluginRuntime;
use ipnet::{Ipv4Net, Ipv6Net};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::net::IpAddr;

const MAX_RESPONSE_SIZE: u64 = 5 * 1024 * 1024;
const DEFAULT_TIMEOUT: u64 = 30;
const MIN_TIMEOUT: u64 = 1;
const MAX_TIMEOUT: u64 = 300;

fn is_ssrf_url(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return true,
    };

    if !matches!(parsed.scheme(), "http" | "https") {
        return true;
    }

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return true,
    };

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    if host == "::1" || host == "[::1]" {
        return true;
    }

    if let Ok(addr) = host.parse::<IpAddr>() {
        return is_private_ip(addr);
    }

    false
}

fn is_private_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => is_private_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_private_ipv6(ipv6),
    }
}

fn is_private_ipv4(ipv4: std::net::Ipv4Addr) -> bool {
    let private_ranges = [
        Ipv4Net::new(std::net::Ipv4Addr::new(127, 0, 0, 0), 8)
            .expect("Invalid IPv4 loopback address range"),
        Ipv4Net::new(std::net::Ipv4Addr::new(10, 0, 0, 0), 8)
            .expect("Invalid IPv4 private network 10.0.0.0/8"),
        Ipv4Net::new(std::net::Ipv4Addr::new(172, 16, 0, 0), 12)
            .expect("Invalid IPv4 private network 172.16.0.0/12"),
        Ipv4Net::new(std::net::Ipv4Addr::new(192, 168, 0, 0), 16)
            .expect("Invalid IPv4 private network 192.168.0.0/16"),
    ];

    private_ranges.iter().any(|range| range.contains(&ipv4))
}

fn is_private_ipv6(ipv6: std::net::Ipv6Addr) -> bool {
    let private_ranges = [
        Ipv6Net::new(std::net::Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7)
            .expect("Invalid IPv6 unique local address range"), // 唯一本地地址范围
        Ipv6Net::new(std::net::Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10)
            .expect("Invalid IPv6 link-local address range"), // 链路本地地址范围
    ];

    private_ranges.iter().any(|range| range.contains(&ipv6))
}

fn lua_error(lua: &Lua, msg: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Nil, Value::String(lua.create_string(msg)?)]))
}

fn lua_success(_lua: &Lua, table: Table) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_vec(vec![Value::Table(table), Value::Nil]))
}

impl PluginRuntime {
    pub(super) fn setup_http_namespace(&self, sl: &Table) -> Result<(), String> {
        let http_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create http table: {}", e))?;

        let plugin_id = self.plugin_id.clone();
        let permissions = self.permissions.clone();

        let pid = plugin_id.clone();
        let perms = permissions.clone();
        let get_fn = self
            .lua
            .create_function(move |lua, args: MultiValue| {
                execute_http_request(lua, &pid, &perms, args, HttpMethod::Get, None)
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
                let args_clone = args.clone();
                let body_arg = args_clone.get(1);
                execute_http_request(lua, &pid, &perms, args, HttpMethod::Post, body_arg)
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
                let args_clone = args.clone();
                let body_arg = args_clone.get(1);
                execute_http_request(lua, &pid, &perms, args, HttpMethod::Put, body_arg)
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
                execute_http_request(lua, &pid, &perms, args, HttpMethod::Delete, None)
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

enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

fn execute_http_request(
    lua: &Lua,
    plugin_id: &str,
    permissions: &[String],
    args: MultiValue,
    method: HttpMethod,
    body_arg: Option<&Value>,
) -> LuaResult<MultiValue> {
    if !permissions.iter().any(|p| p == "network") {
        return Err(mlua::Error::runtime("Permission denied: 'network' permission required"));
    }

    let url = extract_url(args.front())?;

    let api_name = match method {
        HttpMethod::Get => "sl.http.get",
        HttpMethod::Post => "sl.http.post",
        HttpMethod::Put => "sl.http.put",
        HttpMethod::Delete => "sl.http.delete",
    };

    let _ = crate::plugins::api::emit_permission_log(plugin_id, "api_call", api_name, &url);

    if is_ssrf_url(&url) {
        return Err(mlua::Error::runtime(
            "SSRF: Access to internal network, localhost, or non-HTTP(S) addresses is not allowed",
        ));
    }

    let (headers, timeout) = parse_http_options(match method {
        HttpMethod::Get => args.get(1),
        _ => args.get(2),
    })?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .map_err(|e| mlua::Error::runtime(format!("Failed to create HTTP client: {}", e)))?;

    let request = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => {
            let (body_str, is_json) = lua_body_to_string(body_arg)?;
            let mut headers = headers;
            if is_json
                && !headers
                    .iter()
                    .any(|(k, _)| k.to_lowercase() == "content-type")
            {
                headers.push(("Content-Type".to_string(), "application/json".to_string()));
            }
            return send_request(lua, client.post(&url).body(body_str), headers, MAX_RESPONSE_SIZE);
        }
        HttpMethod::Put => {
            let (body_str, is_json) = lua_body_to_string(body_arg)?;
            let mut headers = headers;
            if is_json
                && !headers
                    .iter()
                    .any(|(k, _)| k.to_lowercase() == "content-type")
            {
                headers.push(("Content-Type".to_string(), "application/json".to_string()));
            }
            return send_request(lua, client.put(&url).body(body_str), headers, MAX_RESPONSE_SIZE);
        }
        HttpMethod::Delete => client.delete(&url),
    };

    send_request(lua, request, headers, MAX_RESPONSE_SIZE)
}

fn send_request(
    lua: &Lua,
    request: reqwest::blocking::RequestBuilder,
    headers: Vec<(String, String)>,
    max_size: u64,
) -> LuaResult<MultiValue> {
    let mut request = request;
    for (k, v) in &headers {
        request = request.header(k.as_str(), v.as_str());
    }

    match request.send() {
        Ok(resp) => build_response_table(lua, resp, max_size),
        Err(e) => lua_error(lua, &e.to_string()),
    }
}

fn extract_url(value: Option<&Value>) -> Result<String, mlua::Error> {
    value
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        })
        .ok_or_else(|| mlua::Error::runtime("First argument must be a URL string"))
}

fn parse_http_options(
    options: Option<&Value>,
) -> Result<(Vec<(String, String)>, u64), mlua::Error> {
    let mut headers = Vec::new();
    let mut timeout = DEFAULT_TIMEOUT;

    if let Some(Value::Table(opts)) = options {
        if let Ok(Value::Table(h)) = opts.get::<Value>("headers") {
            for (k, v) in h.pairs::<String, String>().flatten() {
                headers.push((k, v));
            }
        }

        if let Ok(t) = opts.get::<u64>("timeout") {
            if (MIN_TIMEOUT..=MAX_TIMEOUT).contains(&t) {
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
        Some(Value::Table(table)) => {
            let json_val = json_value_from_lua(&Value::Table(table.clone()), 0)?;
            let json_str = serde_json::to_string(&json_val).map_err(|e| {
                mlua::Error::runtime(format!("Failed to serialize body to JSON: {}", e))
            })?;
            Ok((json_str, true))
        }
        Some(Value::Nil) | None => Ok((String::new(), false)),
        _ => Err(mlua::Error::runtime("Body parameter must be a string or table")),
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

    let body_bytes = resp
        .bytes()
        .map_err(|e| mlua::Error::runtime(format!("Failed to read response body: {}", e)))?;

    if body_bytes.len() as u64 > max_size {
        return lua_error(
            lua,
            &format!(
                "Response body too large: {} bytes, limit is {} bytes",
                body_bytes.len(),
                max_size
            ),
        );
    }

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let response_table = lua.create_table()?;
    response_table.set("status", status)?;
    response_table.set("body", body_str)?;
    response_table.set("headers", headers_table)?;

    lua_success(lua, response_table)
}
