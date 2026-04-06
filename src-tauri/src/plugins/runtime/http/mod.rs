use super::shared::json_value_from_lua;
use super::PluginRuntime;
use ipnet::{Ipv4Net, Ipv6Net};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::net::IpAddr;

mod common;
mod request;
mod response;

use common::{
    create_http_function, create_http_table, set_http_function, set_http_table, HttpContext,
};

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
            .expect("Invalid IPv6 unique local address range"),
        Ipv6Net::new(std::net::Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10)
            .expect("Invalid IPv6 link-local address range"),
    ];

    private_ranges.iter().any(|range| range.contains(&ipv6))
}

fn execute_http_request(
    lua: &Lua,
    ctx: &HttpContext,
    args: MultiValue,
    method: request::HttpMethod,
    body_arg: Option<&Value>,
) -> LuaResult<MultiValue> {
    if !ctx.permissions.iter().any(|p| p == "network") {
        return Err(mlua::Error::runtime("Permission denied: 'network' permission required"));
    }

    let url = request::extract_url(args.front())?;
    let api_name = request::api_name(&method);

    let _ = crate::plugins::api::emit_permission_log(&ctx.plugin_id, "api_call", api_name, &url);

    if is_ssrf_url(&url) {
        return Err(mlua::Error::runtime(
            "SSRF: Access to internal network, localhost, or non-HTTP(S) addresses is not allowed",
        ));
    }

    let (headers, timeout) = request::parse_http_options(match method {
        request::HttpMethod::Get => args.get(1),
        _ => args.get(2),
    })?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .map_err(|e| mlua::Error::runtime(format!("Failed to create HTTP client: {}", e)))?;

    let request = match method {
        request::HttpMethod::Get => client.get(&url),
        request::HttpMethod::Post => {
            let (body_str, is_json) = request::lua_body_to_string(body_arg)?;
            let headers = request::with_json_content_type(headers, is_json);
            return response::send_request(
                lua,
                client.post(&url).body(body_str),
                headers,
                MAX_RESPONSE_SIZE,
            );
        }
        request::HttpMethod::Put => {
            let (body_str, is_json) = request::lua_body_to_string(body_arg)?;
            let headers = request::with_json_content_type(headers, is_json);
            return response::send_request(
                lua,
                client.put(&url).body(body_str),
                headers,
                MAX_RESPONSE_SIZE,
            );
        }
        request::HttpMethod::Delete => client.delete(&url),
    };

    response::send_request(lua, request, headers, MAX_RESPONSE_SIZE)
}

impl PluginRuntime {
    pub(super) fn setup_http_namespace(&self, sl: &Table) -> Result<(), String> {
        let http_table = create_http_table(&self.lua)?;
        let ctx = HttpContext::new(self.plugin_id.clone(), self.permissions.clone());

        set_http_function(
            &http_table,
            "get",
            create_http_function(&self.lua, &ctx, request::HttpMethod::Get)?,
            "http.get",
        )?;
        set_http_function(
            &http_table,
            "post",
            create_http_function(&self.lua, &ctx, request::HttpMethod::Post)?,
            "http.post",
        )?;
        set_http_function(
            &http_table,
            "put",
            create_http_function(&self.lua, &ctx, request::HttpMethod::Put)?,
            "http.put",
        )?;
        set_http_function(
            &http_table,
            "delete",
            create_http_function(&self.lua, &ctx, request::HttpMethod::Delete)?,
            "http.delete",
        )?;

        set_http_table(sl, http_table)
    }
}
