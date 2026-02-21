use mlua::{Lua, Result as LuaResult, Value};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

pub(crate) const MAX_RECURSION_DEPTH: usize = 64;

pub(crate) fn json_value_from_lua(value: &Value, depth: usize) -> Result<JsonValue, mlua::Error> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(mlua::Error::runtime("Maximum recursion depth exceeded (64)"));
    }

    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Number((*i).into())),
        Value::Number(n) => Ok(serde_json::Number::from_f64(*n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)),
        Value::String(s) => {
            Ok(JsonValue::String(s.to_str().map(|s| s.to_string()).unwrap_or_default()))
        }
        Value::Table(t) => {
            let mut is_array = true;
            let mut max_index = 0;
            for (k, _) in t.clone().pairs::<Value, Value>().flatten() {
                if let Value::Integer(i) = k {
                    if i > 0 {
                        max_index = max_index.max(i as usize);
                        continue;
                    }
                }
                is_array = false;
                break;
            }

            if is_array && max_index > 0 {
                let mut arr = Vec::with_capacity(max_index);
                for i in 1..=max_index {
                    if let Ok(v) = t.get::<Value>(i) {
                        arr.push(json_value_from_lua(&v, depth + 1)?);
                    } else {
                        arr.push(JsonValue::Null);
                    }
                }
                Ok(JsonValue::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for (k, v) in t.clone().pairs::<String, Value>().flatten() {
                    map.insert(k, json_value_from_lua(&v, depth + 1)?);
                }
                Ok(JsonValue::Object(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}

pub(crate) fn lua_value_from_json(lua: &Lua, value: &JsonValue, depth: usize) -> LuaResult<Value> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(mlua::Error::runtime("Maximum recursion depth exceeded (64)"));
    }

    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, lua_value_from_json(lua, v, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.clone(), lua_value_from_json(lua, v, depth + 1)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub(crate) fn safe_canonicalize_check(
    base_dir: &Path,
    full_path: &Path,
) -> Result<PathBuf, String> {
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析基准目录: {}", e))?;

    if full_path.exists() {
        let canonical = full_path
            .canonicalize()
            .map_err(|e| format!("无法解析路径: {}", e))?;
        if !canonical.starts_with(&canonical_base) {
            return Err("路径必须在允许的目录内".to_string());
        }
        Ok(canonical)
    } else {
        let mut existing_ancestor = full_path.to_path_buf();
        let mut remaining_parts: Vec<std::ffi::OsString> = Vec::new();

        loop {
            if existing_ancestor.exists() {
                break;
            }
            match existing_ancestor.file_name() {
                Some(name) => {
                    remaining_parts.push(name.to_os_string());
                    existing_ancestor.pop();
                }
                None => {
                    return Err("无法找到存在的祖先目录".to_string());
                }
            }
        }

        let canonical_ancestor = existing_ancestor
            .canonicalize()
            .map_err(|e| format!("无法解析祖先目录: {}", e))?;

        if !canonical_ancestor.starts_with(&canonical_base) {
            return Err("路径必须在允许的目录内".to_string());
        }

        let mut result = canonical_ancestor;
        for part in remaining_parts.into_iter().rev() {
            result.push(part);
        }
        Ok(result)
    }
}

pub(super) fn validate_path_static(
    plugin_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    let path = PathBuf::from(relative_path);

    if path.is_absolute() {
        return Err(mlua::Error::runtime("Absolute paths are not allowed"));
    }

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(mlua::Error::runtime("Path cannot contain '..'"));
        }
    }

    let full_path = plugin_dir.join(&path);
    safe_canonicalize_check(plugin_dir, &full_path).map_err(mlua::Error::runtime)
}

pub(super) fn validate_server_path(
    server_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, mlua::Error> {
    let path = PathBuf::from(relative_path);

    if path.is_absolute() {
        return Err(mlua::Error::runtime("不允许使用绝对路径"));
    }

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(mlua::Error::runtime("路径不能包含 '..'"));
        }
    }

    let full_path = server_dir.join(&path);
    safe_canonicalize_check(server_dir, &full_path).map_err(mlua::Error::runtime)
}
