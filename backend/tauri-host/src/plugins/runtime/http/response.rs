use std::io::Read;

use mlua::{Lua, MultiValue, Result as LuaResult};

use super::common::{lua_error, lua_success};

pub(super) fn send_request(
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

fn build_response_table(
    lua: &Lua,
    resp: reqwest::blocking::Response,
    max_size: u64,
) -> LuaResult<MultiValue> {
    let status = resp.status().as_u16();

    let headers_table = lua.create_table()?;
    for (name, value) in resp.headers() {
        if let Ok(v) = value.to_str() {
            headers_table.set(name.as_str().to_string(), v.to_string())?;
        }
    }

    let body_bytes = read_response_body(resp, max_size)
        .map_err(|e| mlua::Error::runtime(format!("Failed to read response body: {}", e)))?;
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let response_table = lua.create_table()?;
    response_table.set("status", status)?;
    response_table.set("body", body_str)?;
    response_table.set("headers", headers_table)?;

    lua_success(response_table)
}

fn read_response_body(
    mut resp: reqwest::blocking::Response,
    max_size: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut body = Vec::new();
    let mut chunk = [0_u8; 8192];

    loop {
        let read = resp.read(&mut chunk)?;
        if read == 0 {
            break;
        }

        body.extend_from_slice(&chunk[..read]);
        if body.len() as u64 > max_size {
            return Err(format!(
                "Response body too large: {} bytes, limit is {} bytes",
                body.len(),
                max_size
            )
            .into());
        }
    }

    Ok(body)
}
