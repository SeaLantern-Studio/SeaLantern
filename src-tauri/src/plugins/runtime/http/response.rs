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
