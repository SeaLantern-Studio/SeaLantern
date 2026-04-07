use super::{json_value_from_lua, DEFAULT_TIMEOUT, MAX_TIMEOUT, MIN_TIMEOUT};
use mlua::Value;

pub(super) struct HttpOptions {
    pub(super) headers: Vec<(String, String)>,
    pub(super) timeout: u64,
}

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

    pub(super) fn api_name(self) -> &'static str {
        match self {
            Self::Get => "sl.http.get",
            Self::Post => "sl.http.post",
            Self::Put => "sl.http.put",
            Self::Delete => "sl.http.delete",
        }
    }

    pub(super) fn accepts_body(self) -> bool {
        matches!(self, Self::Post | Self::Put)
    }
}

pub(super) fn extract_url(value: Option<&Value>) -> Result<String, mlua::Error> {
    let url = value
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
            _ => None,
        })
        .ok_or_else(|| mlua::Error::runtime("First argument must be a URL string"))?;

    url::Url::parse(&url).map_err(|e| mlua::Error::runtime(format!("Invalid URL: {}", e)))?;

    Ok(url)
}

pub(super) fn option_arg(method: HttpMethod, args: &[Value]) -> Option<&Value> {
    if method.accepts_body() {
        args.get(2)
    } else {
        args.get(1)
    }
}

pub(super) fn parse_http_options(options: Option<&Value>) -> Result<HttpOptions, mlua::Error> {
    let mut headers = Vec::new();
    let mut timeout = DEFAULT_TIMEOUT;

    let Some(options) = options else {
        return Ok(HttpOptions { headers, timeout });
    };

    let Value::Table(opts) = options else {
        return Err(mlua::Error::runtime("Options parameter must be a table when provided"));
    };

    match opts.get::<Value>("headers")? {
        Value::Nil => {}
        Value::Table(h) => {
            for pair in h.pairs::<String, String>() {
                let (key, value) = pair.map_err(|_| {
                    mlua::Error::runtime(
                        "Options.headers must be a table of string keys and string values",
                    )
                })?;
                headers.push((key, value));
            }
        }
        _ => {
            return Err(mlua::Error::runtime("Options.headers must be a table when provided"));
        }
    }

    match opts.get::<Value>("timeout")? {
        Value::Nil => {}
        Value::Integer(t) if t >= MIN_TIMEOUT as i64 && t <= MAX_TIMEOUT as i64 => {
            timeout = t as u64;
        }
        Value::Integer(_) | Value::Number(_) => {
            return Err(mlua::Error::runtime(format!(
                "Options.timeout must be an integer between {} and {} seconds",
                MIN_TIMEOUT, MAX_TIMEOUT
            )));
        }
        _ => {
            return Err(mlua::Error::runtime("Options.timeout must be an integer when provided"));
        }
    }

    Ok(HttpOptions { headers, timeout })
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
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
    {
        headers.push(("Content-Type".to_string(), "application/json".to_string()));
    }

    headers
}
