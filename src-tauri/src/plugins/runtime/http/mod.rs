use super::shared::json_value_from_lua;
use super::PluginRuntime;
use mlua::{Lua, MultiValue, Result as LuaResult, Table};
use reqwest::redirect::Policy;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::time::Duration;

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

fn is_blocked_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost") || host.eq_ignore_ascii_case("localhost.")
}

pub(crate) fn validate_ssrf_url(url: &str) -> Result<(), mlua::Error> {
    let parsed = url::Url::parse(url)
        .map_err(|_| mlua::Error::runtime("SSRF: only valid HTTP(S) URLs are allowed"))?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(mlua::Error::runtime("SSRF: Access to non-HTTP(S) addresses is not allowed"));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| mlua::Error::runtime("SSRF: URL must contain a host"))?;

    if is_blocked_host(host) {
        return Err(mlua::Error::runtime("SSRF: Access to localhost is not allowed"));
    }

    if let Ok(addr) = host.parse::<IpAddr>() {
        if is_private_ip(addr) {
            return Err(mlua::Error::runtime(
                "SSRF: Access to internal network addresses is not allowed",
            ));
        }
        return Ok(());
    }

    let port = parsed.port_or_known_default().ok_or_else(|| {
        mlua::Error::runtime("SSRF: URL must use a known port for the selected scheme")
    })?;

    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| mlua::Error::runtime(format!("SSRF: failed to resolve host: {}", e)))?;

    let mut resolved_any = false;
    for socket_addr in addrs {
        resolved_any = true;
        if is_private_ip(socket_addr.ip()) {
            return Err(mlua::Error::runtime(
                "SSRF: Access to internal network addresses is not allowed",
            ));
        }
    }

    if !resolved_any {
        return Err(mlua::Error::runtime("SSRF: host resolution returned no usable addresses"));
    }

    Ok(())
}

fn is_private_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => is_private_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_private_ipv6(ipv6),
    }
}

fn is_private_ipv4(ipv4: Ipv4Addr) -> bool {
    ipv4.is_private()
        || ipv4.is_loopback()
        || ipv4.is_link_local()
        || ipv4.is_unspecified()
        || ipv4.is_broadcast()
        || ipv4.is_documentation()
}

fn is_private_ipv6(ipv6: Ipv6Addr) -> bool {
    let segments = ipv6.segments();
    let is_documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;

    ipv6.is_loopback()
        || ipv6.is_unique_local()
        || ipv6.is_unicast_link_local()
        || ipv6.is_unspecified()
        || is_documentation
}

fn create_http_client(timeout: u64) -> Result<reqwest::blocking::Client, mlua::Error> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .redirect(Policy::none())
        .build()
        .map_err(|e| mlua::Error::runtime(format!("Failed to create HTTP client: {}", e)))
}

fn execute_http_request(
    lua: &Lua,
    ctx: &HttpContext,
    args: MultiValue,
    method: request::HttpMethod,
) -> LuaResult<MultiValue> {
    if !ctx.permissions.iter().any(|p| p == "network") {
        return Err(mlua::Error::runtime("Permission denied: 'network' permission required"));
    }

    let args_vec: Vec<_> = args.into_vec();
    let url = request::extract_url(args_vec.first())?;
    let api_name = method.api_name();

    let _ = crate::plugins::api::emit_permission_log(&ctx.plugin_id, "api_call", api_name, &url);

    validate_ssrf_url(&url)?;

    let options = request::parse_http_options(request::option_arg(method, &args_vec))?;
    let client = create_http_client(options.timeout)?;

    let result = if method.accepts_body() {
        let body_arg = args_vec.get(1);
        let (body_str, is_json) = request::lua_body_to_string(body_arg)?;
        let headers = request::with_json_content_type(options.headers, is_json);
        let request = match method {
            request::HttpMethod::Post => client.post(&url),
            request::HttpMethod::Put => client.put(&url),
            _ => unreachable!("checked by accepts_body"),
        }
        .body(body_str);

        response::send_request(lua, request, headers, MAX_RESPONSE_SIZE)
    } else {
        let request = match method {
            request::HttpMethod::Get => client.get(&url),
            request::HttpMethod::Delete => client.delete(&url),
            _ => unreachable!("non-body method branch"),
        };

        response::send_request(lua, request, options.headers, MAX_RESPONSE_SIZE)
    };

    result
}

impl PluginRuntime {
    pub(super) fn setup_http_namespace(&self, sl: &Table) -> Result<(), String> {
        let http_table = create_http_table(&self.lua)?;
        let ctx = HttpContext::new(self.plugin_id.clone(), self.permissions.clone());

        for (name, method) in [
            ("get", request::HttpMethod::Get),
            ("post", request::HttpMethod::Post),
            ("put", request::HttpMethod::Put),
            ("delete", request::HttpMethod::Delete),
        ] {
            set_http_function(
                &http_table,
                name,
                create_http_function(&self.lua, &ctx, method)?,
                &format!("http.{}", name),
            )?;
        }

        set_http_table(sl, http_table)
    }
}
