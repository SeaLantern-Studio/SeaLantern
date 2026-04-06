use super::{json_value_from_lua, DEFAULT_TIMEOUT, MAX_TIMEOUT, MIN_TIMEOUT};
use mlua::Value;

#[derive(Clone, Copy)]
pub(super) enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
        }
    }
}

pub(super) fn api_name(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "sl.http.get",
        HttpMethod::Post => "sl.http.post",
        HttpMethod::Put => "sl.http.put",
        HttpMethod::Delete => "sl.http.delete",
    }
}

pub(super) fn extract_url(value: Option<&Value>) -> Result<String, mlua::Error> {
    value
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        })
        .ok_or_else(|| mlua::Error::runtime("First argument must be a URL string"))
}

pub(super) fn parse_http_options(
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

pub(super) fn lua_body_to_string(body: Option<&Value>) -> Result<(String, bool), mlua::Error> {
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

pub(super) fn with_json_content_type(
    mut headers: Vec<(String, String)>,
    is_json: bool,
) -> Vec<(String, String)> {
    if is_json
        && !headers
            .iter()
            .any(|(k, _)| k.to_lowercase() == "content-type")
    {
        headers.push(("Content-Type".to_string(), "application/json".to_string()));
    }

    headers
}
